use std::time::Duration;
use sqlx::PgPool;
use tokio::time::sleep;
use serde::{Deserialize, Serialize};
use crate::models::CollateralStatus;

#[derive(Clone)]
pub struct CollateralIndexer {
    db_pool: PgPool,
    rpc_url: String,
    contract_id: String,
}

impl CollateralIndexer {
    pub fn new(db_pool: PgPool, rpc_url: String, contract_id: String) -> Self {
        Self {
            db_pool,
            rpc_url,
            contract_id,
        }
    }

    pub async fn start(&self) {
        tracing::info!("Starting Collateral Indexer for contract {}", self.contract_id);
        
        // Spawn the event loop
        let indexer = self.clone();
        tokio::spawn(async move {
            indexer.run_event_loop().await;
        });
    }

    async fn run_event_loop(&self) {
        let mut last_cursor = "0".to_string(); // Start from beginning or load from DB
        
        loop {
            match self.fetch_events(&last_cursor).await {
                Ok((events, new_cursor)) => {
                    for event in events {
                        if let Err(e) = self.process_event(event).await {
                            tracing::error!("Failed to process event: {}", e);
                            // In a real system, we might retry or DLQ this event
                        }
                    }
                    last_cursor = new_cursor;
                }
                Err(e) => {
                    tracing::error!("Error fetching events: {}", e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
            
            // Polling interval
            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn fetch_events(&self, cursor: &str) -> Result<(Vec<CollateralEvent>, String), String> {
        // Mock implementation
        // In real code: call Soroban RPC getEvents(start_ledger: cursor)
        
        // Return empty list mostly, but occasionally could return a mock event if we wanted to test
        // For now, keep it simple and clean.
        Ok((vec![], cursor.to_string()))
    }

    async fn process_event(&self, event: CollateralEvent) -> Result<(), String> {
        match event {
            CollateralEvent::Registered { collateral_id, tx_hash, .. } => {
                tracing::info!("Processing Registered event for {}", collateral_id);
                // We assume the service already created the record, but if we are "syncing from chain",
                // we might need to UPSERT here.
                // For now, let's update the status to ensure it matches chain.
                sqlx::query(
                    "UPDATE collateral SET status = $1, tx_hash = COALESCE(tx_hash, $2) WHERE collateral_id = $3"
                )
                .bind(CollateralStatus::Active)
                .bind(tx_hash)
                .bind(collateral_id)
                .execute(&self.db_pool)
                .await
                .map_err(|e| e.to_string())?;
            }
            CollateralEvent::Locked { collateral_id } => {
                tracing::info!("Processing Locked event for {}", collateral_id);
                sqlx::query(
                    "UPDATE collateral SET locked = true, status = $1 WHERE collateral_id = $2"
                )
                .bind(CollateralStatus::Locked)
                .bind(collateral_id)
                .execute(&self.db_pool)
                .await
                .map_err(|e| e.to_string())?;
            }
            CollateralEvent::Unlocked { collateral_id } => {
                 tracing::info!("Processing Unlocked event for {}", collateral_id);
                 sqlx::query(
                    "UPDATE collateral SET locked = false, status = $1 WHERE collateral_id = $2"
                )
                .bind(CollateralStatus::Active)
                .bind(collateral_id)
                .execute(&self.db_pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollateralEvent {
    Registered {
        collateral_id: String,
        owner: String,
        face_value: i64,
        tx_hash: String,
    },
    Locked {
        collateral_id: String,
    },
    Unlocked {
        collateral_id: String,
    },
}
