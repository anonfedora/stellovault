use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

mod handlers;
mod types;

use handlers::EventHandler;
use types::GetEventsResponse;
use crate::websocket::WsState;

pub struct IndexerService {
    rpc_url: String,
    pool: PgPool,
    contracts: HashMap<String, String>, // Name -> ID
    client: Client,
    ws_state: WsState,
}

impl IndexerService {
    pub fn new(
        rpc_url: String,
        pool: PgPool,
        contracts: HashMap<String, String>,
        ws_state: WsState,
    ) -> Self {
        Self {
            rpc_url,
            pool: pool.clone(),
            contracts,
            client: Client::new(),
            ws_state,
        }
    }

    pub async fn start(self: Arc<Self>) {
        tracing::info!("Starting Soroban Indexer Service...");
        
        // Spawn a task for each contract
        let handles: Vec<_> = self.contracts.iter().map(|(name, id)| {
            let name = name.clone();
            let id = id.clone();
            let rpc_url = self.rpc_url.clone();
            let pool = self.pool.clone();
            let client = self.client.clone();
            let ws_state = self.ws_state.clone();
            
            // Each indexer gets its own handler instance
            let handler = EventHandler::new(pool.clone(), Some(ws_state));
            
            tokio::spawn(async move {
                let mut indexer = ContractIndexer {
                    name,
                    contract_id: id,
                    rpc_url,
                    pool,
                    client,
                    handler,
                };
                indexer.run().await;
            })
        }).collect();

        // We act as a supervisor here, or just exit and let tasks run.
        // If we await, this blocks the caller (main).
        // Usually start() spawns tasks and returns or awaits forever.
        // Given existing main.rs spawns event listener, we can just await here if main spawns us.
        // But main.rs calls `tokio::spawn(async move { ... })`.
        // So we can await handles.
        
        for handle in handles {
            let _ = handle.await;
        }
    }
}

struct ContractIndexer {
    name: String,
    contract_id: String,
    rpc_url: String,
    pool: PgPool,
    client: Client,
    handler: EventHandler,
}

impl ContractIndexer {
    async fn run(&mut self) {
        tracing::info!("Indexer started for {} ({})", self.name, self.contract_id);
        
        loop {
            if let Err(e) = self.process_batch().await {
                tracing::error!("Error indexing {}: {}", self.name, e);
                sleep(Duration::from_secs(5)).await;
            }
            sleep(Duration::from_secs(2)).await;
        }
    }

    async fn process_batch(&mut self) -> Result<()> {
        let cursor = self.get_last_cursor().await?;
        
        let response = self.fetch_events(&cursor).await?;
        
        if response.events.is_empty() {
             return Ok(());
        }

        tracing::debug!("Fetched {} events for {}", response.events.len(), self.name);

        let mut last_cursor = cursor.clone();
        let mut max_ledger = 0;

        for event in &response.events {
            self.handler.handle_event(event, &self.name).await?;
            last_cursor = event.paging_token.clone();
            max_ledger = event.ledger;
        }

        // Update cursor
        if last_cursor != cursor {
            self.save_cursor(&last_cursor, max_ledger).await?;
        }

        Ok(())
    }

    async fn fetch_events(&self, cursor: &str) -> Result<GetEventsResponse> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getEvents",
            "params": {
                "startLedger": if cursor.is_empty() { json!(1) } else { serde_json::Value::Null }, 
                "filters": [
                    {
                        "type": "contract",
                        "contractIds": [self.contract_id]
                    }
                ],
                "pagination": {
                    "cursor": if cursor.is_empty() { serde_json::Value::Null } else { json!(cursor) },
                    "limit": 100
                }
            }
        });

        let resp = self.client.post(&self.rpc_url)
            .json(&payload)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if let Some(err) = resp.get("error") {
            return Err(anyhow::anyhow!("RPC Error: {:?}", err));
        }

        let result = resp.get("result").ok_or(anyhow::anyhow!("No result in RPC response"))?;
        let events_response: GetEventsResponse = serde_json::from_value(result.clone())?;

        Ok(events_response)
    }

    async fn get_last_cursor(&self) -> Result<String> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT last_cursor FROM indexer_state WHERE contract_id = $1"
        )
        .bind(&self.contract_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0).unwrap_or_default())
    }

    async fn save_cursor(&self, cursor: &str, ledger: u64) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO indexer_state (contract_id, last_cursor, last_seen_ledger, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (contract_id) 
            DO UPDATE SET last_cursor = EXCLUDED.last_cursor, last_seen_ledger = EXCLUDED.last_seen_ledger, updated_at = NOW()
            "#
        )
        .bind(&self.contract_id)
        .bind(cursor)
        .bind(ledger as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
