//! Event listener for Soroban contract events

use anyhow::Result;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

use crate::escrow::{EscrowEvent, EscrowStatus};
use crate::escrow_service::EscrowService;
use crate::websocket::WsState;

/// Soroban event from Horizon API
#[derive(Debug, Deserialize, Clone)]
pub struct SorobanEvent {
    pub _id: String,
    #[serde(rename = "type")]
    pub _event_type: String,
    pub _contract_id: String,
    pub topic: Vec<String>,
    pub _value: String,
    pub _ledger: u64,
}

/// Event listener service
pub struct EventListener {
    _horizon_url: String,
    contract_id: String,
    escrow_service: Arc<EscrowService>,
    ws_state: WsState,
    db_pool: PgPool,
    _last_cursor: Option<String>,
}

impl EventListener {
    /// Create new event listener
    pub fn new(
        horizon_url: String,
        contract_id: String,
        escrow_service: Arc<EscrowService>,
        ws_state: WsState,
        db_pool: PgPool,
    ) -> Self {
        Self {
            _horizon_url: horizon_url,
            contract_id,
            escrow_service,
            ws_state,
            db_pool,
            _last_cursor: None,
        }
    }

    /// Start listening for events
    pub async fn start(mut self) {
        tracing::info!("Starting event listener for contract {}", self.contract_id);

        loop {
            if let Err(e) = self.poll_events().await {
                tracing::error!("Error polling events: {}", e);
            }

            // Poll every 5 seconds
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    /// Poll for new events from Horizon API
    async fn poll_events(&mut self) -> Result<()> {
        // TODO: Implement actual Horizon API polling
        // For now, simulate event polling from database changes

        // Check for status changes in database that haven't been broadcast
        let recent_updates = self.get_recent_updates().await?;

        for (escrow_id, status) in recent_updates {
            let event = match status {
                EscrowStatus::Active => EscrowEvent::Activated { escrow_id },
                EscrowStatus::Released => EscrowEvent::Released { escrow_id },
                EscrowStatus::Cancelled => EscrowEvent::Cancelled { escrow_id },
                EscrowStatus::TimedOut => EscrowEvent::TimedOut { escrow_id },
                EscrowStatus::Disputed => EscrowEvent::Disputed {
                    escrow_id,
                    reason: "Dispute detected".to_string(),
                },
                _ => continue,
            };

            // Process event
            self.process_event(event).await?;
        }

        Ok(())
    }

    /// Process a single event
    async fn process_event(&self, event: EscrowEvent) -> Result<()> {
        // Update database via service
        self.escrow_service.process_escrow_event(event.clone()).await?;

        // Broadcast to WebSocket clients
        self.ws_state.broadcast_event(event).await;

        Ok(())
    }

    /// Parse Soroban event into EscrowEvent
    #[allow(dead_code)]
    fn parse_soroban_event(&self, event: SorobanEvent) -> Option<EscrowEvent> {
        // Parse topic to determine event type
        if event.topic.is_empty() {
            return None;
        }

        let event_type = &event.topic[0];

        match event_type.as_str() {
            "esc_crtd" => {
                // Escrow created event
                // TODO: Parse buyer_id, seller_id from event data
                Some(EscrowEvent::Created {
                    escrow_id: 0, // Parse from event
                    buyer_id: uuid::Uuid::nil(),
                    seller_id: uuid::Uuid::nil(),
                })
            }
            "esc_act" => {
                // Escrow activated
                Some(EscrowEvent::Activated {
                    escrow_id: 0, // Parse from event
                })
            }
            "esc_rel" => {
                // Escrow released
                Some(EscrowEvent::Released {
                    escrow_id: 0, // Parse from event
                })
            }
            _ => {
                tracing::warn!("Unknown event type: {}", event_type);
                None
            }
        }
    }

    /// Get recent database updates (simulation)
    async fn get_recent_updates(&self) -> Result<Vec<(i64, EscrowStatus)>> {
        let updates = sqlx::query_as::<_, (i64, EscrowStatus)>(
            r#"
            SELECT escrow_id, status 
            FROM escrows 
            WHERE updated_at > NOW() - INTERVAL '10 seconds'
            ORDER BY updated_at DESC
            "#,
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(updates.into_iter().map(|(id, status)| (id as i64, status)).collect())
    }
}

/// Background job for timeout detection
pub async fn timeout_detector(
    escrow_service: Arc<EscrowService>,
    ws_state: WsState,
) {
    tracing::info!("Starting timeout detector");

    loop {
        // Check for timeouts every minute
        tokio::time::sleep(Duration::from_secs(60)).await;

        match escrow_service.detect_timeouts().await {
            Ok(timed_out_escrows) => {
                for escrow_id in timed_out_escrows {
                    let event = EscrowEvent::TimedOut { escrow_id };
                    ws_state.broadcast_event(event).await;
                    tracing::info!("Escrow {} timed out", escrow_id);
                }
            }
            Err(e) => {
                tracing::error!("Error detecting timeouts: {}", e);
            }
        }
    }
}
