//! WebSocket server for real-time escrow updates

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

use crate::escrow::EscrowEvent;

/// WebSocket server state
#[derive(Clone)]
pub struct WsState {
    /// Broadcast channel for escrow events
    pub tx: broadcast::Sender<EscrowEvent>,
    /// Connected clients registry
    pub clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
}

/// Client connection information
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub client_id: String,
    pub subscribed_escrows: Vec<i64>,
}

/// Client message types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    Subscribe { escrow_ids: Vec<i64> },
    Unsubscribe { escrow_ids: Vec<i64> },
    Ping,
}

/// Server message types
#[derive(Debug, Serialize)]
#[allow(dead_code)]
#[serde(tag = "type")]
enum ServerMessage {
    Event { event: EscrowEvent },
    Subscribed { escrow_ids: Vec<i64> },
    Unsubscribed { escrow_ids: Vec<i64> },
    Pong,
    Error { message: String },
}

impl WsState {
    /// Create new WebSocket state
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self {
            tx,
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Broadcast an escrow event to all connected clients
    pub async fn broadcast_event(&self, event: EscrowEvent) {
        if let Err(e) = self.tx.send(event.clone()) {
            tracing::error!("Failed to broadcast event: {}", e);
        }
    }

    /// Register a new client
    async fn register_client(&self, client_id: String) {
        let mut clients = self.clients.write().await;
        clients.insert(
            client_id.clone(),
            ClientInfo {
                client_id,
                subscribed_escrows: vec![],
            },
        );
    }

    /// Unregister a client
    async fn unregister_client(&self, client_id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
        tracing::info!("Client {} disconnected", client_id);
    }

    /// Update client subscriptions
    async fn update_subscriptions(&self, client_id: &str, escrow_ids: Vec<i64>) {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            client.subscribed_escrows = escrow_ids;
        }
    }
}

/// WebSocket handler - upgrades HTTP connection to WebSocket
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<WsState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: WsState) {
    let client_id = Uuid::new_v4().to_string();
    state.register_client(client_id.clone()).await;

    let (mut sender, mut receiver) = socket.split();

    // Internal channel for sending messages from recv_task to sender
    let (internal_tx, mut internal_rx) = mpsc::channel::<ServerMessage>(32);

    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();
    let client_id_clone = client_id.clone();
    let state_clone = state.clone();

    // Spawn task to forward broadcast events and internal messages to this client
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle broadcast events
                Ok(event) = rx.recv() => {
                    let clients = state_clone.clients.read().await;
                    if let Some(client_info) = clients.get(&client_id_clone) {
                        let should_send = match &event {
                            EscrowEvent::Created { escrow_id, .. }
                            | EscrowEvent::Activated { escrow_id }
                            | EscrowEvent::Released { escrow_id }
                            | EscrowEvent::Cancelled { escrow_id }
                            | EscrowEvent::TimedOut { escrow_id }
                            | EscrowEvent::Disputed { escrow_id, .. }
                            | EscrowEvent::StatusUpdated { escrow_id, .. } => {
                                client_info.subscribed_escrows.is_empty()
                                    || client_info.subscribed_escrows.contains(escrow_id)
                            }
                        };

                        if should_send {
                            let msg = ServerMessage::Event { event };
                            if let Ok(text) = serde_json::to_string(&msg) {
                                if sender.send(Message::Text(text)).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
                // Handle internal messages (confirmations, pongs)
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
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    match client_msg {
                        ClientMessage::Subscribe { escrow_ids } => {
                            state_recv
                                .update_subscriptions(&client_id_recv, escrow_ids.clone())
                                .await;
                            let response = ServerMessage::Subscribed { escrow_ids };
                            let _ = internal_tx.send(response).await;
                            tracing::info!("Client {} subscribed", client_id_recv);
                        }
                        ClientMessage::Unsubscribe { escrow_ids } => {
                            // Remove specific subscriptions
                            let clients = state_recv.clients.read().await;
                            if let Some(client_info) = clients.get(&client_id_recv) {
                                let mut current = client_info.subscribed_escrows.clone();
                                current.retain(|id| !escrow_ids.contains(id));
                                drop(clients); // Release lock
                                state_recv
                                    .update_subscriptions(&client_id_recv, current)
                                    .await;
                            }
                            let response = ServerMessage::Unsubscribed { escrow_ids };
                            let _ = internal_tx.send(response).await;
                            tracing::info!("Client {} unsubscribed", client_id_recv);
                        }
                        ClientMessage::Ping => {
                            // Respond with pong (keepalive)
                            tracing::debug!("Ping from client {}", client_id_recv);
                            let _ = internal_tx.send(ServerMessage::Pong).await;
                        }
                    }
                }
            } else if let Message::Close(_) = msg {
                break;
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

// Re-export futures traits for split() and send()
use futures_util::{SinkExt, StreamExt};
