use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration as StdDuration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use crate::config::contracts::ContractsConfig;

const POLL_INTERVAL_SECONDS: u64 = 10;
const MAX_BACKOFF_SECONDS: u64 = 60;
const HTTP_TIMEOUT_SECONDS: u64 = 15;
const EVENTS_PAGE_LIMIT: u64 = 200;
const MAX_EVENTS_PER_CYCLE: usize = 2_000;

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
    event_id: String,
    event_name: String,
    tx_hash: Option<String>,
    payload: Value,
}

#[derive(Debug)]
pub enum StartError {
    NoContractsConfigured,
}

impl EventMonitoringService {
    pub async fn from_env() -> Self {
        let state_path = std::env::var("INDEXER_STATE_FILE")
            .unwrap_or_else(|_| "server/.indexer_state.json".to_string());
        let mirror_db_path = std::env::var("INDEXER_DB_FILE")
            .unwrap_or_else(|_| "server/.indexer_mirror_db.json".to_string());
        let rpc_url = std::env::var("SOROBAN_RPC_URL")
            .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());
        let http_timeout_seconds = std::env::var("INDEXER_HTTP_TIMEOUT_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(HTTP_TIMEOUT_SECONDS);

        let last_processed_ledger = read_indexer_state(&PathBuf::from(&state_path))
            .await
            .map(|state| state.last_processed_ledger)
            .unwrap_or(0);

        Self {
            rpc_url,
            contracts: ContractsConfig::from_env(),
            indexer_state_path: PathBuf::from(state_path),
            mirror_db_path: PathBuf::from(mirror_db_path),
            http: Client::builder()
                .timeout(StdDuration::from_secs(http_timeout_seconds))
                .build()
                .unwrap_or_else(|_| Client::new()),
            cursor: Arc::new(Mutex::new(last_processed_ledger)),
        }
    }

    pub async fn start(self) -> Result<(), StartError> {
        if self.contracts.monitored_contract_ids().is_empty() {
            return Err(StartError::NoContractsConfigured);
        }

        info!("Soroban event indexer started");
        let mut failure_count: u32 = 0;

        loop {
            match self.poll_once().await {
                Ok(()) => {
                    failure_count = 0;
                    sleep(Duration::from_secs(POLL_INTERVAL_SECONDS)).await;
                }
                Err(err) => {
                    failure_count = failure_count.saturating_add(1);
                    let exponential = POLL_INTERVAL_SECONDS
                        .saturating_mul(2u64.saturating_pow(failure_count.min(6)));
                    let backoff_seconds = exponential.min(MAX_BACKOFF_SECONDS);
                    error!(
                        error = %err,
                        failure_count,
                        backoff_seconds,
                        "event indexer poll cycle failed"
                    );
                    sleep(Duration::from_secs(backoff_seconds)).await;
                }
            }
        }
    }

    async fn poll_once(&self) -> Result<(), String> {
        let latest_ledger = self.fetch_latest_ledger().await?;

        let from_ledger = {
            let cursor_guard = self.cursor.lock().await;
            cursor_guard.saturating_add(1)
        };
        if from_ledger > latest_ledger {
            return Ok(());
        }

        // Limit the range per cycle to keep requests and processing predictable.
        let to_ledger = latest_ledger.min(from_ledger + 200);
        let (events, max_seen_ledger) = self.fetch_events(from_ledger, to_ledger).await?;
        let parsed_events: Vec<ParsedEvent> = events.into_iter().filter_map(parse_event).collect();
        let mut mirror = read_mirror_db(&self.mirror_db_path).await.unwrap_or_default();

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

        self.process_collateral_events(&mut mirror, &collateral_events).await?;
        self.process_escrow_events(&mut mirror, &escrow_events).await?;
        self.process_loan_events(&mut mirror, &loan_events).await?;
        self.process_governance_events(&mut mirror, &governance_events).await?;
        persist_mirror_db(&self.mirror_db_path, &mirror).await?;

        let safe_ledger = if parsed_events.is_empty() {
            to_ledger
        } else {
            max_seen_ledger.max(from_ledger)
        };
        {
            let mut cursor_guard = self.cursor.lock().await;
            *cursor_guard = safe_ledger;
        }
        persist_indexer_state(
            &self.indexer_state_path,
            &IndexerState {
                last_processed_ledger: safe_ledger,
            },
        )
        .await?;

        info!(
            from_ledger,
            to_ledger = safe_ledger,
            processed_events = parsed_events.len(),
            "Indexer cycle complete"
        );

        Ok(())
    }

    async fn process_collateral_events(
        &self,
        mirror: &mut MirrorDb,
        events: &[ParsedEvent],
    ) -> Result<(), String> {
        if events.is_empty() {
            return Ok(());
        }

        for event in events {
            let key = entity_key(event, "collateral");
            mirror.collateral.insert(key, to_record(event));
        }

        Ok(())
    }

    async fn process_escrow_events(
        &self,
        mirror: &mut MirrorDb,
        events: &[ParsedEvent],
    ) -> Result<(), String> {
        if events.is_empty() {
            return Ok(());
        }

        for event in events {
            let key = entity_key(event, "escrow");
            let record = to_record(event);
            mirror.escrows.insert(key, record.clone());
            if !record_exists(&mirror.ws_broadcast_log, &record) {
                mirror.ws_broadcast_log.push(record);
            }
        }

        Ok(())
    }

    async fn process_loan_events(
        &self,
        mirror: &mut MirrorDb,
        events: &[ParsedEvent],
    ) -> Result<(), String> {
        if events.is_empty() {
            return Ok(());
        }

        for event in events {
            let key = entity_key(event, "loan");
            mirror.loans.insert(key, to_record(event));
        }

        Ok(())
    }

    async fn process_governance_events(
        &self,
        mirror: &mut MirrorDb,
        events: &[ParsedEvent],
    ) -> Result<(), String> {
        if events.is_empty() {
            return Ok(());
        }

        for event in events {
            let record = to_record(event);
            if !record_exists(&mirror.governance_audit_log, &record) {
                mirror.governance_audit_log.push(record);
            }
        }

        Ok(())
    }

    async fn fetch_latest_ledger(&self) -> Result<u64, String> {
        let response = self
            .rpc_call("getLatestLedger", json!({}))
            .await
            .map_err(|err| format!("getLatestLedger failed: {err}"))?;

        if let Some(rpc_error) = response.pointer("/error") {
            return Err(format!("getLatestLedger RPC error: {}", rpc_error));
        }

        response
            .pointer("/result/sequence")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "missing latest ledger sequence in RPC response".to_string())
    }

    async fn fetch_events(
        &self,
        start_ledger: u64,
        end_ledger: u64,
    ) -> Result<(Vec<Value>, u64), String> {
        let mut filters = vec![];
        for contract_id in self.contracts.monitored_contract_ids() {
            filters.push(json!({
                "type": "contract",
                "contractIds": [contract_id],
            }));
        }

        let mut all_events: Vec<Value> = vec![];
        let mut cursor: Option<String> = None;
        let mut max_seen_ledger = 0_u64;

        loop {
            let mut pagination = json!({ "limit": EVENTS_PAGE_LIMIT });
            if let Some(cursor_value) = &cursor {
                pagination["cursor"] = Value::String(cursor_value.clone());
            }

            let payload = if cursor.is_some() {
                json!({
                    "pagination": pagination,
                    "filters": filters,
                    "xdrFormat": "json",
                })
            } else {
                json!({
                    "startLedger": start_ledger,
                    "endLedger": end_ledger,
                    "pagination": pagination,
                    "filters": filters,
                    "xdrFormat": "json",
                })
            };

            let response = self
                .rpc_call("getEvents", payload)
                .await
                .map_err(|err| format!("getEvents failed: {err}"))?;

            if let Some(rpc_error) = response.pointer("/error") {
                return Err(format!("getEvents RPC error: {}", rpc_error));
            }

            let page_events = response
                .pointer("/result/events")
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default();

            for event in page_events {
                max_seen_ledger = max_seen_ledger.max(extract_event_ledger(&event));
                all_events.push(event);

                if all_events.len() >= MAX_EVENTS_PER_CYCLE {
                    return Ok((all_events, max_seen_ledger));
                }
            }

            cursor = response
                .pointer("/result/cursor")
                .and_then(|value| value.as_str())
                .filter(|value| !value.is_empty())
                .map(ToString::to_string);

            if cursor.is_none() {
                break;
            }
        }

        Ok((all_events, max_seen_ledger))
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

    let event_name = extract_topic_symbol(raw.pointer("/topic").or_else(|| raw.pointer("/topics")))
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

    let event_id = raw
        .pointer("/id")
        .or_else(|| raw.pointer("/pagingToken"))
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();

    let payload = raw
        .pointer("/value")
        .or_else(|| raw.pointer("/data"))
        .cloned()
        .unwrap_or(Value::Null);

    Some(ParsedEvent {
        ledger,
        contract_id,
        event_id,
        event_name,
        tx_hash,
        payload,
    })
}

fn extract_topic_symbol(value: Option<&Value>) -> Option<String> {
    let topics = value?.as_array()?;
    let first_topic = topics.first()?;

    first_topic
        .pointer("/symbol")
        .or_else(|| first_topic.pointer("/value/symbol"))
        .or_else(|| first_topic.pointer("/value"))
        .or_else(|| first_topic.pointer("/scvSymbol"))
        .and_then(|symbol| symbol.as_str())
        .map(ToString::to_string)
}

fn entity_key(event: &ParsedEvent, kind: &str) -> String {
    if !event.event_id.is_empty() {
        return format!("{kind}:{}", event.event_id);
    }

    format!(
        "{}:{}:{}:{}",
        kind,
        event.tx_hash.clone().unwrap_or_else(|| "unknown-tx".to_string()),
        event.ledger,
        event.event_name
    )
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
    write_atomic(path, payload.as_bytes()).await
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
    write_atomic(path, payload.as_bytes()).await
}

async fn write_atomic(path: &PathBuf, payload: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| err.to_string())?;
    }

    let temp_path = PathBuf::from(format!("{}.tmp", path.display()));
    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|err| err.to_string())?;
    file.write_all(payload).await.map_err(|err| err.to_string())?;
    file.sync_all().await.map_err(|err| err.to_string())?;
    drop(file);

    tokio::fs::rename(&temp_path, path)
        .await
        .map_err(|err| err.to_string())
}

fn extract_event_ledger(event: &Value) -> u64 {
    event
        .pointer("/ledger")
        .or_else(|| event.pointer("/ledgerSequence"))
        .or_else(|| event.pointer("/ledger_sequence"))
        .and_then(|value| value.as_u64())
        .unwrap_or(0)
}

fn record_exists(records: &[EventRecord], candidate: &EventRecord) -> bool {
    records.iter().any(|record| {
        record.tx_hash == candidate.tx_hash
            && record.ledger == candidate.ledger
            && record.event_name == candidate.event_name
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_event_prefers_topic_symbol_name() {
        let raw = json!({
            "id": "event-1",
            "contractId": "contract-a",
            "ledger": 42,
            "txHash": "tx-1",
            "type": "contract",
            "topics": [{ "symbol": "EscrowCreated" }],
            "value": "AAAA..."
        });

        let parsed = parse_event(raw).expect("expected event to parse");
        assert_eq!(parsed.event_name, "EscrowCreated");
        assert_eq!(parsed.event_id, "event-1");
    }

    #[test]
    fn entity_key_uses_event_id_when_present() {
        let event = ParsedEvent {
            ledger: 10,
            contract_id: "contract-a".to_string(),
            event_id: "paging-123".to_string(),
            event_name: "LoanIssued".to_string(),
            tx_hash: Some("tx-abc".to_string()),
            payload: Value::String("AAAA".to_string()),
        };

        assert_eq!(entity_key(&event, "loan"), "loan:paging-123");
    }

    #[test]
    fn extract_topic_symbol_supports_nested_json_shape() {
        let topics = json!([{
            "value": {
                "symbol": "LoanRepaid"
            }
        }]);
        let symbol = extract_topic_symbol(Some(&topics));
        assert_eq!(symbol.as_deref(), Some("LoanRepaid"));
    }

    #[tokio::test]
    async fn process_escrow_events_deduplicates_broadcast_log() {
        let service = EventMonitoringService {
            rpc_url: "http://localhost".to_string(),
            contracts: ContractsConfig::from_env(),
            indexer_state_path: PathBuf::from("/tmp/indexer_state_test.json"),
            mirror_db_path: PathBuf::from("/tmp/indexer_db_test.json"),
            http: Client::new(),
            cursor: Arc::new(Mutex::new(0)),
        };

        let event = ParsedEvent {
            ledger: 99,
            contract_id: "escrow".to_string(),
            event_id: "evt-1".to_string(),
            event_name: "EscrowCreated".to_string(),
            tx_hash: Some("tx-1".to_string()),
            payload: Value::String("AAAA".to_string()),
        };

        let mut mirror = MirrorDb::default();
        let events = vec![event.clone(), event];
        service
            .process_escrow_events(&mut mirror, &events)
            .await
            .expect("processing should succeed");

        assert_eq!(mirror.ws_broadcast_log.len(), 1);
    }
}
