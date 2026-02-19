use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use crate::config::contracts::ContractsConfig;

const POLL_INTERVAL_SECONDS: u64 = 10;

#[derive(Clone)]
pub struct EventMonitoringService {
    rpc_url: String,
    contracts: ContractsConfig,
    indexer_state_path: PathBuf,
    mirror_db_path: PathBuf,
    http: Client,
    cursor: Arc<Mutex<u64>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct IndexerState {
    last_processed_ledger: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct EventRecord {
    ledger: u64,
    contract_id: String,
    event_name: String,
    tx_hash: Option<String>,
    payload: Value,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct MirrorDb {
    collateral: HashMap<String, EventRecord>,
    escrows: HashMap<String, EventRecord>,
    loans: HashMap<String, EventRecord>,
    governance_audit_log: Vec<EventRecord>,
    ws_broadcast_log: Vec<EventRecord>,
}

#[derive(Debug, Clone)]
struct ParsedEvent {
    ledger: u64,
    contract_id: String,
    event_name: String,
    tx_hash: Option<String>,
    payload: Value,
}

impl EventMonitoringService {
    pub async fn from_env() -> Self {
        let state_path = std::env::var("INDEXER_STATE_FILE")
            .unwrap_or_else(|_| "server/.indexer_state.json".to_string());
        let mirror_db_path = std::env::var("INDEXER_DB_FILE")
            .unwrap_or_else(|_| "server/.indexer_mirror_db.json".to_string());
        let rpc_url = std::env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());

        let last_processed_ledger = read_indexer_state(&PathBuf::from(&state_path))
            .await
            .map(|state| state.last_processed_ledger)
            .unwrap_or(0);

        Self {
            rpc_url,
            contracts: ContractsConfig::from_env(),
            indexer_state_path: PathBuf::from(state_path),
            mirror_db_path: PathBuf::from(mirror_db_path),
            http: Client::new(),
            cursor: Arc::new(Mutex::new(last_processed_ledger)),
        }
    }

    pub async fn start(self) {
        if self.contracts.monitored_contract_ids().is_empty() {
            warn!("Indexer disabled: no contract IDs set in environment");
            return;
        }

        info!("Soroban event indexer started");

        loop {
            if let Err(err) = self.poll_once().await {
                error!(error = %err, "event indexer poll cycle failed");
            }

            sleep(Duration::from_secs(POLL_INTERVAL_SECONDS)).await;
        }
    }

    async fn poll_once(&self) -> Result<(), String> {
        let latest_ledger = self.fetch_latest_ledger().await?;

        let mut cursor_guard = self.cursor.lock().await;
        let from_ledger = cursor_guard.saturating_add(1);
        if from_ledger > latest_ledger {
            return Ok(());
        }

        // Limit the range per cycle to keep requests and processing predictable.
        let to_ledger = latest_ledger.min(from_ledger + 200);
        let events = self.fetch_events(from_ledger, to_ledger).await?;
        let parsed_events: Vec<ParsedEvent> = events.into_iter().filter_map(parse_event).collect();

        let collateral_contract = self.contracts.collateral_contract_id.clone();
        let escrow_contract = self.contracts.escrow_contract_id.clone();
        let loan_contract = self.contracts.loan_contract_id.clone();
        let governance_contract = self.contracts.governance_contract_id.clone();

        let collateral_events: Vec<ParsedEvent> = parsed_events
            .iter()
            .filter(|event| {
                event.contract_id == collateral_contract
                    && matches!(
                        event.event_name.as_str(),
                        "CollateralDeposited" | "CollateralReleased"
                    )
            })
            .cloned()
            .collect();

        let escrow_events: Vec<ParsedEvent> = parsed_events
            .iter()
            .filter(|event| {
                event.contract_id == escrow_contract
                    && matches!(
                        event.event_name.as_str(),
                        "EscrowCreated" | "EscrowSettled" | "EscrowExpired"
                    )
            })
            .cloned()
            .collect();

        let loan_events: Vec<ParsedEvent> = parsed_events
            .iter()
            .filter(|event| {
                event.contract_id == loan_contract
                    && matches!(
                        event.event_name.as_str(),
                        "LoanIssued" | "LoanRepaid" | "LoanDefaulted"
                    )
            })
            .cloned()
            .collect();

        let governance_events: Vec<ParsedEvent> = parsed_events
            .iter()
            .filter(|event| {
                governance_contract
                    .as_ref()
                    .map(|id| event.contract_id == *id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        self.process_collateral_events(&collateral_events).await?;
        self.process_escrow_events(&escrow_events).await?;
        self.process_loan_events(&loan_events).await?;
        self.process_governance_events(&governance_events).await?;

        *cursor_guard = to_ledger;
        persist_indexer_state(
            &self.indexer_state_path,
            &IndexerState {
                last_processed_ledger: to_ledger,
            },
        )
        .await?;

        info!(from_ledger, to_ledger, processed_events = parsed_events.len(), "Indexer cycle complete");

        Ok(())
    }

    async fn process_collateral_events(&self, events: &[ParsedEvent]) -> Result<(), String> {
        if events.is_empty() {
            return Ok(());
        }

        let mut mirror = read_mirror_db(&self.mirror_db_path).await.unwrap_or_default();

        for event in events {
            let key = entity_key(event, "collateral");
            mirror.collateral.insert(key, to_record(event));
        }

        persist_mirror_db(&self.mirror_db_path, &mirror).await
    }

    async fn process_escrow_events(&self, events: &[ParsedEvent]) -> Result<(), String> {
        if events.is_empty() {
            return Ok(());
        }

        let mut mirror = read_mirror_db(&self.mirror_db_path).await.unwrap_or_default();

        for event in events {
            let key = entity_key(event, "escrow");
            let record = to_record(event);
            mirror.escrows.insert(key, record.clone());
            mirror.ws_broadcast_log.push(record);
        }

        persist_mirror_db(&self.mirror_db_path, &mirror).await
    }

    async fn process_loan_events(&self, events: &[ParsedEvent]) -> Result<(), String> {
        if events.is_empty() {
            return Ok(());
        }

        let mut mirror = read_mirror_db(&self.mirror_db_path).await.unwrap_or_default();

        for event in events {
            let key = entity_key(event, "loan");
            mirror.loans.insert(key, to_record(event));
        }

        persist_mirror_db(&self.mirror_db_path, &mirror).await
    }

    async fn process_governance_events(&self, events: &[ParsedEvent]) -> Result<(), String> {
        if events.is_empty() {
            return Ok(());
        }

        let mut mirror = read_mirror_db(&self.mirror_db_path).await.unwrap_or_default();

        for event in events {
            mirror.governance_audit_log.push(to_record(event));
        }

        persist_mirror_db(&self.mirror_db_path, &mirror).await
    }

    async fn fetch_latest_ledger(&self) -> Result<u64, String> {
        let response = self
            .rpc_call("getLatestLedger", json!({}))
            .await
            .map_err(|err| format!("getLatestLedger failed: {err}"))?;

        response
            .pointer("/result/sequence")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "missing latest ledger sequence in RPC response".to_string())
    }

    async fn fetch_events(&self, start_ledger: u64, end_ledger: u64) -> Result<Vec<Value>, String> {
        let mut filters = vec![];
        for contract_id in self.contracts.monitored_contract_ids() {
            filters.push(json!({
                "type": "contract",
                "contractIds": [contract_id],
            }));
        }

        let payload = json!({
            "startLedger": start_ledger,
            "endLedger": end_ledger,
            "pagination": { "limit": 200 },
            "filters": filters,
        });

        let response = self
            .rpc_call("getEvents", payload)
            .await
            .map_err(|err| format!("getEvents failed: {err}"))?;

        Ok(response
            .pointer("/result/events")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default())
    }

    async fn rpc_call(&self, method: &str, params: Value) -> Result<Value, reqwest::Error> {
        self.http
            .post(&self.rpc_url)
            .json(&json!({
                "jsonrpc": "2.0",
                "id": "stellovault-indexer",
                "method": method,
                "params": params,
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await
    }
}

fn parse_event(raw: Value) -> Option<ParsedEvent> {
    let contract_id = raw
        .pointer("/contractId")
        .or_else(|| raw.pointer("/contract_id"))
        .or_else(|| raw.pointer("/contract/id"))
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();

    if contract_id.is_empty() {
        return None;
    }

    let event_name = raw
        .pointer("/type")
        .or_else(|| raw.pointer("/eventType"))
        .and_then(|value| value.as_str())
        .map(|s| s.to_string())
        .or_else(|| extract_topic_symbol(raw.pointer("/topic").or_else(|| raw.pointer("/topics"))))
        .unwrap_or_else(|| "UnknownEvent".to_string());

    let ledger = raw
        .pointer("/ledger")
        .or_else(|| raw.pointer("/ledgerSequence"))
        .and_then(|value| value.as_u64())
        .unwrap_or(0);

    let tx_hash = raw
        .pointer("/txHash")
        .or_else(|| raw.pointer("/tx_hash"))
        .and_then(|value| value.as_str())
        .map(ToString::to_string);

    let payload = raw
        .pointer("/value")
        .or_else(|| raw.pointer("/data"))
        .cloned()
        .unwrap_or(Value::Null);

    Some(ParsedEvent {
        ledger,
        contract_id,
        event_name,
        tx_hash,
        payload,
    })
}

fn extract_topic_symbol(value: Option<&Value>) -> Option<String> {
    let topics = value?.as_array()?;
    topics
        .first()
        .and_then(|topic| topic.pointer("/symbol").or_else(|| topic.pointer("/value")))
        .and_then(|symbol| symbol.as_str())
        .map(ToString::to_string)
}

fn entity_key(event: &ParsedEvent, kind: &str) -> String {
    event
        .payload
        .pointer("/id")
        .or_else(|| event.payload.pointer("/key"))
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            format!(
                "{}:{}:{}",
                kind,
                event.tx_hash.clone().unwrap_or_else(|| "unknown-tx".to_string()),
                event.ledger
            )
        })
}

fn to_record(event: &ParsedEvent) -> EventRecord {
    EventRecord {
        ledger: event.ledger,
        contract_id: event.contract_id.clone(),
        event_name: event.event_name.clone(),
        tx_hash: event.tx_hash.clone(),
        payload: event.payload.clone(),
    }
}

async fn read_indexer_state(path: &PathBuf) -> Result<IndexerState, String> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|err| err.to_string())?;
    serde_json::from_str(&content).map_err(|err| err.to_string())
}

async fn persist_indexer_state(path: &PathBuf, state: &IndexerState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| err.to_string())?;
    }

    let payload = serde_json::to_string_pretty(state).map_err(|err| err.to_string())?;
    tokio::fs::write(path, payload)
        .await
        .map_err(|err| err.to_string())
}

async fn read_mirror_db(path: &PathBuf) -> Result<MirrorDb, String> {
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|err| err.to_string())?;
    serde_json::from_str(&content).map_err(|err| err.to_string())
}

async fn persist_mirror_db(path: &PathBuf, db: &MirrorDb) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| err.to_string())?;
    }

    let payload = serde_json::to_string_pretty(db).map_err(|err| err.to_string())?;
    tokio::fs::write(path, payload)
        .await
        .map_err(|err| err.to_string())
}
