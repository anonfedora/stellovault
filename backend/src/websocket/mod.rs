//! High-Performance WebSocket Event Dispatcher
//! 
//! Features:
//! - Room-based subscriptions (e.g., subscribe:escrow:123)
//! - Heartbeat system to prune dead connections
//! - Event buffering for 30-second reconnection replay
//! - Scalable architecture for 1000+ concurrent connections

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::escrow::EscrowEvent as EscrowEventType;

/// WebSocket escrow events - wrapper around escrow events with proper serialization
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event")]
pub enum EscrowEvent {
    Created { escrow_id: i64, data: serde_json::Value },
    Activated { escrow_id: i64 },
    Released { escrow_id: i64, released_by: String },
    Cancelled { escrow_id: i64, reason: String },
    TimedOut { escrow_id: i64 },
    Disputed { escrow_id: i64, dispute_reason: String },
    StatusUpdated { escrow_id: i64, old_status: String, new_status: String },
}

impl From<EscrowEventType> for EscrowEvent {
    fn from(e: EscrowEventType) -> Self {
        match e {
            EscrowEventType::Created { escrow_id, buyer_id, seller_id } => EscrowEvent::Created {
                escrow_id,
                data: serde_json::json!({ "buyer_id": buyer_id, "seller_id": seller_id }),
            },
            EscrowEventType::Activated { escrow_id } => EscrowEvent::Activated { escrow_id },
            EscrowEventType::Released { escrow_id } => EscrowEvent::Released { 
                escrow_id,
                released_by: "contract".to_string() 
            },
            EscrowEventType::Refunded { escrow_id } => EscrowEvent::Released {
                escrow_id,
                released_by: "refund".to_string(),
            },
            EscrowEventType::Cancelled { escrow_id } => EscrowEvent::Cancelled { 
                escrow_id, 
                reason: "cancelled_by_party".to_string() 
            },
            EscrowEventType::TimedOut { escrow_id } => EscrowEvent::TimedOut { escrow_id },
            EscrowEventType::Disputed { escrow_id, reason } => EscrowEvent::Disputed { 
                escrow_id, 
                dispute_reason: reason 
            },
            EscrowEventType::StatusUpdated { escrow_id, status } => EscrowEvent::StatusUpdated { 
                escrow_id, 
                old_status: "unknown".to_string(), 
                new_status: format!("{:?}", status) 
            },
            EscrowEventType::Refunded { escrow_id } => EscrowEvent::Cancelled {
                escrow_id,
                reason: "refunded".to_string(),
            },
        }
    }
}

/// WebSocket server state - optimized for high concurrency
#[derive(Clone)]
pub struct WsState {
    /// Broadcast channel for escrow events (large buffer for high throughput)
    pub tx: broadcast::Sender<WsEvent>,
    /// Connected clients registry with room subscriptions
    pub clients: Arc<RwLock<HashMap<String, ClientConnection>>>,
    /// Room to client mapping for efficient routing
    pub rooms: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Event buffer for reconnection replay (30 seconds)
    pub event_buffer: Arc<RwLock<EventBuffer>>,
    /// Connection metadata for heartbeat tracking
    pub conn_metadata: Arc<RwLock<HashMap<String, ConnectionMetadata>>>,
}

/// Room subscription format: "type:id" e.g., "escrow:123", "loan:456"
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Room {
    pub room_type: String,
    pub room_id: String,
}

impl Room {
    /// Parse room string like "escrow:123" into Room struct
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            Some(Room {
                room_type: parts[0].to_string(),
                room_id: parts[1].to_string(),
            })
        } else {
            None
        }
    }
    
    /// Convert to string format "type:id"
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.room_type, self.room_id)
    }
}

/// Generic WebSocket event structure supporting multiple event types
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WsEvent {
    Escrow(EscrowEvent),
    Loan(LoanEvent),
    System(SystemEvent),
}

/// Loan events
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event")]
pub enum LoanEvent {
    Created { loan_id: i64, data: serde_json::Value },
    Funded { loan_id: i64 },
    Repaid { loan_id: i64, amount: String },
    Defaulted { loan_id: i64 },
    StatusUpdated { loan_id: i64, old_status: String, new_status: String },
}

/// System events
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event")]
pub enum SystemEvent {
    Connected { client_id: String },
    Heartbeat,
    RoomJoined { room: String },
    RoomLeft { room: String },
}

/// Connection metadata for heartbeat tracking
#[derive(Debug, Clone)]
pub struct ConnectionMetadata {
    pub client_id: String,
    pub connected_at: Instant,
    pub last_heartbeat: Instant,
    pub subscribed_rooms: HashSet<String>,
    pub reconnect_token: Option<String>,
}

/// Client connection information
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub client_id: String,
    pub subscribed_rooms: HashSet<String>,
    pub sender: Option<mpsc::Sender<ServerMessage>>,
}

/// Buffered event with timestamp for replay
#[derive(Debug, Clone)]
struct BufferedEvent {
    event: WsEvent,
    timestamp: Instant,
    event_id: u64,
}

/// Circular event buffer for reconnection replay (30 seconds)
struct EventBuffer {
    events: Vec<BufferedEvent>,
    max_duration: Duration,
    max_events: usize,
    next_event_id: u64,
}

impl EventBuffer {
    fn new(max_duration_secs: u64, max_events: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_events),
            max_duration: Duration::from_secs(max_duration_secs),
            max_events,
            next_event_id: 0,
        }
    }
    
    /// Add event to buffer, removing expired events
    fn push(&mut self, event: WsEvent) {
        let now = Instant::now();
        let event_id = self.next_event_id;
        self.next_event_id += 1;
        
        // Remove expired events older than max_duration
        let cutoff = now - self.max_duration;
        self.events.retain(|e| e.timestamp > cutoff);
        
        // Add new event
        self.events.push(BufferedEvent {
            event,
            timestamp: now,
            event_id,
        });
        
        // Trim if exceeds max
        while self.events.len() > self.max_events {
            self.events.remove(0);
        }
    }
    
    /// Get events since a given event ID for reconnection replay
    fn get_events_since(&self, since_id: u64) -> Vec<WsEvent> {
        self.events
            .iter()
            .filter(|e| e.event_id > since_id)
            .map(|e| e.event.clone())
            .collect()
    }
    
    /// Get events for specific rooms since a given event ID
    fn get_events_for_rooms(&self, rooms: &HashSet<String>, since_id: u64) -> Vec<WsEvent> {
        self.events
            .iter()
            .filter(|e| e.event_id > since_id && Self::event_matches_rooms(&e.event, rooms))
            .map(|e| e.event.clone())
            .collect()
    }
    
    fn event_matches_rooms(event: &WsEvent, rooms: &HashSet<String>) -> bool {
        match event {
            WsEvent::Escrow(e) => {
                let room_id = match e {
                    EscrowEvent::Created { escrow_id, .. } => escrow_id.to_string(),
                    EscrowEvent::Activated { escrow_id } => escrow_id.to_string(),
                    EscrowEvent::Released { escrow_id, .. } => escrow_id.to_string(),
                    EscrowEvent::Cancelled { escrow_id, .. } => escrow_id.to_string(),
                    EscrowEvent::TimedOut { escrow_id } => escrow_id.to_string(),
                    EscrowEvent::Disputed { escrow_id, .. } => escrow_id.to_string(),
                    EscrowEvent::StatusUpdated { escrow_id, .. } => escrow_id.to_string(),
                };
                rooms.contains(&format!("escrow:{}", room_id))
            }
            WsEvent::Loan(e) => {
                let room_id = match e {
                    LoanEvent::Created { loan_id, .. } => loan_id.to_string(),
                    LoanEvent::Funded { loan_id } => loan_id.to_string(),
                    LoanEvent::Repaid { loan_id, .. } => loan_id.to_string(),
                    LoanEvent::Defaulted { loan_id } => loan_id.to_string(),
                    LoanEvent::StatusUpdated { loan_id, .. } => loan_id.to_string(),
                };
                rooms.contains(&format!("loan:{}", room_id))
            }
            WsEvent::System(_) => false,
        }
    }
}

/// Client message types - enhanced with room subscriptions
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Subscribe to rooms: { "type": "subscribe", "rooms": ["escrow:123", "loan:456"] }
    Subscribe { rooms: Vec<String> },
    /// Unsubscribe from rooms
    Unsubscribe { rooms: Vec<String> },
    /// Ping for heartbeat
    Ping { timestamp: Option<u64> },
    /// Reconnect with replay token to get missed events
    Reconnect { reconnect_token: String, last_event_id: Option<u64> },
}

/// Server message types
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Event notification
    Event { event: WsEvent },
    /// Subscription confirmed
    Subscribed { rooms: Vec<String> },
    /// Unsubscription confirmed
    Unsubscribed { rooms: Vec<String> },
    /// Pong response
    Pong { timestamp: u64, server_time: u64 },
    /// Error message
    Error { code: String, message: String },
    /// Replay events after reconnect
    Replay { events: Vec<WsEvent>, last_event_id: u64 },
    /// Welcome message with client info
    Welcome { client_id: String, reconnect_token: String },
}

impl WsState {
    /// Create new high-performance WebSocket state
    pub fn new() -> Self {
        // Large broadcast channel for high throughput (1000+ connections)
        let (tx, _rx) = broadcast::channel(1000);
        Self {
            tx,
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            event_buffer: Arc::new(RwLock::new(EventBuffer::new(30, 10000))),
            conn_metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Broadcast an event to all relevant rooms and buffer it
    pub async fn broadcast_event(&self, event: WsEvent) {
        // Buffer the event for reconnection replay
        {
            let mut buffer = self.event_buffer.write().await;
            buffer.push(event.clone());
        }
        
        // Broadcast to all subscribers
        if let Err(e) = self.tx.send(event.clone()) {
            tracing::error!("Failed to broadcast event: {}", e);
        }
    }
    
    /// Broadcast event to specific room only
    pub async fn broadcast_to_room(&self, room: &str, event: WsEvent) {
        let rooms = self.rooms.read().await;
        if let Some(client_ids) = rooms.get(room) {
            let buffer = self.event_buffer.read().await;
            let event_id = buffer.next_event_id;
            drop(buffer);
            
            for client_id in client_ids {
                if let Some(client) = self.clients.read().await.get(client_id) {
                    if let Some(sender) = &client.sender {
                        let msg = ServerMessage::Event { event: event.clone() };
                        let _ = sender.try_send(msg);
                    }
                }
            }
        }
        
        // Also buffer for reconnection replay
        let mut buffer = self.event_buffer.write().await;
        buffer.push(event);
    }
    
    /// Join a room - add client to room mapping
    async fn join_room(&self, client_id: &str, room: &str) {
        // Add client to room
        {
            let mut rooms = self.rooms.write().await;
            rooms.entry(room.to_string()).or_default().insert(client_id.to_string());
        }
        
        // Add room to client's subscriptions
        {
            let mut clients = self.clients.write().await;
            if let Some(client) = clients.get_mut(client_id) {
                client.subscribed_rooms.insert(room.to_string());
            }
        }
        
        // Update metadata
        {
            let mut metadata = self.conn_metadata.write().await;
            if let Some(conn) = metadata.get_mut(client_id) {
                conn.subscribed_rooms.insert(room.to_string());
            }
        }
        
        tracing::debug!("Client {} joined room {}", client_id, room);
    }
    
    /// Leave a room
    async fn leave_room(&self, client_id: &str, room: &str) {
        // Remove from room
        {
            let mut rooms = self.rooms.write().await;
            if let Some(client_ids) = rooms.get_mut(room) {
                client_ids.remove(client_id);
                if client_ids.is_empty() {
                    rooms.remove(room);
                }
            }
        }
        
        // Remove room from client's subscriptions
        {
            let mut clients = self.clients.write().await;
            if let Some(client) = clients.get_mut(client_id) {
                client.subscribed_rooms.remove(room);
            }
        }
        
        // Update metadata
        {
            let mut metadata = self.conn_metadata.write().await;
            if let Some(conn) = metadata.get_mut(client_id) {
                conn.subscribed_rooms.remove(room);
            }
        }
        
        tracing::debug!("Client {} left room {}", client_id, room);
    }
    
    /// Register a new client
    async fn register_client(&self, client_id: String) -> String {
        let reconnect_token = Uuid::new_v4().to_string();
        
        // Create client connection
        let (tx, _rx) = mpsc::channel(100);
        let mut clients = self.clients.write().await;
        clients.insert(
            client_id.clone(),
            ClientConnection {
                client_id: client_id.clone(),
                subscribed_rooms: HashSet::new(),
                sender: Some(tx),
            },
        );
        
        // Store metadata
        let mut metadata = self.conn_metadata.write().await;
        metadata.insert(
            client_id.clone(),
            ConnectionMetadata {
                client_id: client_id.clone(),
                connected_at: Instant::now(),
                last_heartbeat: Instant::now(),
                subscribed_rooms: HashSet::new(),
                reconnect_token: Some(reconnect_token.clone()),
            },
        );
        
        tracing::info!("Client {} registered, reconnect_token: {}", client_id, reconnect_token);
        reconnect_token
    }
    
    /// Unregister a client
    async fn unregister_client(&self, client_id: &str) {
        // Get rooms to clean up
        let rooms_to_remove: Vec<String>;
        {
            let metadata = self.conn_metadata.read().await;
            rooms_to_remove = metadata
                .get(client_id)
                .map(|m| m.subscribed_rooms.iter().cloned().collect())
                .unwrap_or_default();
        }
        
        // Remove from all rooms
        for room in &rooms_to_remove {
            self.leave_room(client_id, room).await;
        }
        
        // Remove from clients
        {
            let mut clients = self.clients.write().await;
            clients.remove(client_id);
        }
        
        // Remove metadata (but keep reconnect token for reconnection window)
        {
            let mut metadata = self.conn_metadata.write().await;
            if let Some(conn) = metadata.get_mut(client_id) {
                // Clear sender to prevent message sends
                if let Some(client) = self.clients.write().await.get_mut(client_id) {
                    client.sender = None;
                }
            }
        }
        
        tracing::info!("Client {} disconnected", client_id);
    }
    
    /// Update heartbeat for client
    async fn update_heartbeat(&self, client_id: &str) {
        let mut metadata = self.conn_metadata.write().await;
        if let Some(conn) = metadata.get_mut(client_id) {
            conn.last_heartbeat = Instant::now();
        }
    }
    
    /// Get events for reconnection replay
    async fn get_replay_events(&self, client_id: &str, last_event_id: Option<u64>) -> Vec<WsEvent> {
        let since_id = last_event_id.unwrap_or(0);
        
        // Get rooms this client was subscribed to
        let rooms: HashSet<String>;
        {
            let metadata = self.conn_metadata.read().await;
            rooms = metadata
                .get(client_id)
                .map(|m| m.subscribed_rooms.clone())
                .unwrap_or_default();
        }
        
        // Get buffered events for those rooms
        let buffer = self.event_buffer.read().await;
        if rooms.is_empty() {
            buffer.get_events_since(since_id)
        } else {
            buffer.get_events_for_rooms(&rooms, since_id)
        }
    }
    
    /// Get client count
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }
    
    /// Get room count
    pub async fn room_count(&self) -> usize {
        self.rooms.read().await.len()
    }
}

/// WebSocket handler - upgrades HTTP connection to WebSocket
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<WsState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle WebSocket connection - high-performance implementation
async fn handle_socket(socket: WebSocket, state: WsState) {
    let client_id = Uuid::new_v4().to_string();
    let reconnect_token = state.register_client(client_id.clone()).await;

    let (mut sender, mut receiver) = socket.split();

    // Create channel for sending messages to this client
    let (internal_tx, mut internal_rx) = mpsc::channel::<ServerMessage>(100);

    // Store sender in client registry
    {
        let mut clients = state.clients.write().await;
        if let Some(client) = clients.get_mut(&client_id) {
            client.sender = Some(internal_tx.clone());
        }
    }

    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();
    let state_clone = state.clone();
    let client_id_clone = client_id.clone();

    // Spawn task to forward broadcast events and internal messages
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle broadcast events - filter by rooms
                Ok(event) = rx.recv() => {
                    let clients = state_clone.clients.read().await;
                    if let Some(client_info) = clients.get(&client_id_clone) {
                        // Check if event matches client's rooms
                        if WsState::should_send_event(&event, &client_info.subscribed_rooms) {
                            let msg = ServerMessage::Event { event };
                            if let Ok(text) = serde_json::to_string(&msg) {
                                if sender.send(Message::Text(text)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
                // Handle internal messages
                Some(msg) = internal_rx.recv() => {
                    if let Ok(text) = serde_json::to_string(&msg) {
                        if sender.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                }
                else => break,
            }
        }
    });

    // Handle incoming messages from client
    let state_recv = state.clone();
    let client_id_recv = client_id.clone();
    let reconnect_token_clone = reconnect_token.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        WsState::handle_client_message(
                            &state_recv,
                            &client_id_recv,
                            client_msg,
                            &internal_tx,
                            &reconnect_token_clone,
                        )
                        .await;
                    }
                }
                Message::Ping(data) => {
                    // Respond to ping with pong
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let ping_ts = if data.len() >= 8 {
                        let arr: [u8; 8] = data[..8].try_into().unwrap_or_default();
                        u64::from_le_bytes(arr)
                    } else {
                        0
                    };
                    let _ = internal_tx.send(ServerMessage::Pong {
                        timestamp: ping_ts,
                        server_time: ts,
                    }).await;
                    state_recv.update_heartbeat(&client_id_recv).await;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    // Clean up
    state.unregister_client(&client_id).await;
}

impl WsState {
    fn should_send_event(event: &WsEvent, rooms: &HashSet<String>) -> bool {
        if rooms.is_empty() {
            return true; // Global subscription - receive all
        }
        
        match event {
            WsEvent::Escrow(e) => {
                let room_id = match e {
                    EscrowEvent::Created { escrow_id, .. } => escrow_id.to_string(),
                    EscrowEvent::Activated { escrow_id } => escrow_id.to_string(),
                    EscrowEvent::Released { escrow_id, .. } => escrow_id.to_string(),
                    EscrowEvent::Cancelled { escrow_id, .. } => escrow_id.to_string(),
                    EscrowEvent::TimedOut { escrow_id } => escrow_id.to_string(),
                    EscrowEvent::Disputed { escrow_id, .. } => escrow_id.to_string(),
                    EscrowEvent::StatusUpdated { escrow_id, .. } => escrow_id.to_string(),
                };
                rooms.contains(&format!("escrow:{}", room_id))
            }
            WsEvent::Loan(e) => {
                let room_id = match e {
                    LoanEvent::Created { loan_id, .. } => loan_id.to_string(),
                    LoanEvent::Funded { loan_id } => loan_id.to_string(),
                    LoanEvent::Repaid { loan_id, .. } => loan_id.to_string(),
                    LoanEvent::Defaulted { loan_id } => loan_id.to_string(),
                    LoanEvent::StatusUpdated { loan_id, .. } => loan_id.to_string(),
                };
                rooms.contains(&format!("loan:{}", room_id))
            }
            WsEvent::System(_) => false, // System events are sent individually
        }
    }
    
    async fn handle_client_message(
        state: &WsState,
        client_id: &str,
        msg: ClientMessage,
        sender: &mpsc::Sender<ServerMessage>,
        reconnect_token: &str,
    ) {
        match msg {
            ClientMessage::Subscribe { rooms } => {
                for room in &rooms {
                    if Room::from_string(room).is_some() {
                        state.join_room(client_id, room).await;
                    }
                }
                let _ = sender.send(ServerMessage::Subscribed { rooms }).await;
            }
            ClientMessage::Unsubscribe { rooms } => {
                for room in &rooms {
                    state.leave_room(client_id, room).await;
                }
                let _ = sender.send(ServerMessage::Unsubscribed { rooms }).await;
            }
            ClientMessage::Ping { timestamp } => {
                let now_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let _ = sender.send(ServerMessage::Pong {
                    timestamp: timestamp.unwrap_or(0),
                    server_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                }).await;
                state.update_heartbeat(client_id).await;
            }
            ClientMessage::Reconnect { reconnect_token: token, last_event_id } => {
                if token == reconnect_token {
                    // Valid reconnect - replay missed events
                    let events = state.get_replay_events(client_id, last_event_id).await;
                    let last_id = last_event_id.unwrap_or(0) + events.len() as u64;
                    let _ = sender.send(ServerMessage::Replay { events, last_event_id: last_id }).await;
                    
                    // Re-register with rooms from metadata
                    let metadata = state.conn_metadata.read().await;
                    if let Some(conn) = metadata.get(client_id) {
                        for room in &conn.subscribed_rooms {
                            state.join_room(client_id, room).await;
                        }
                    }
                } else {
                    let _ = sender.send(ServerMessage::Error {
                        code: "INVALID_TOKEN".to_string(),
                        message: "Invalid or expired reconnect token".to_string(),
                    }).await;
                }
            }
        }
    }
}

/// Background task to prune dead connections
pub async fn heartbeat_pruner(state: WsState, interval_secs: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
    
    loop {
        interval.tick().await;
        
        let now = Instant::now();
        let timeout = Duration::from_secs(60); // 60 seconds without heartbeat = dead
        
        let mut dead_clients: Vec<String> = Vec::new();
        
        {
            let metadata = state.conn_metadata.read().await;
            for (client_id, conn) in metadata.iter() {
                if now.duration_since(conn.last_heartbeat) > timeout {
                    dead_clients.push(client_id.clone());
                }
            }
        }
        
        for client_id in &dead_clients {
            tracing::warn!("Pruning dead client: {}", client_id);
            state.unregister_client(client_id).await;
        }
        
        if !dead_clients.is_empty() {
            tracing::info!("Pruned {} dead connections", dead_clients.len());
        }
    }
}

// Re-export futures traits
use futures_util::{SinkExt, StreamExt};
