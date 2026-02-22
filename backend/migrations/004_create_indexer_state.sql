-- Create table to track indexer state for each contract
CREATE TABLE IF NOT EXISTS indexer_state (
    contract_id TEXT PRIMARY KEY,
    last_cursor TEXT NOT NULL,
    last_seen_ledger BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Trigger to update updated_at
CREATE TRIGGER update_indexer_state_updated_at BEFORE UPDATE ON indexer_state
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
