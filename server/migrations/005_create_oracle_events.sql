-- I'm creating the oracle events table to track all confirmations and audit history.

-- Create oracle data type enum
CREATE TYPE oracle_data_type AS ENUM ('shipping', 'iot', 'manual');

-- Create oracle event status enum
CREATE TYPE oracle_event_status AS ENUM ('pending', 'confirmed', 'aggregated', 'disputed', 'rejected');

-- Oracle events table - stores all oracle confirmations
CREATE TABLE IF NOT EXISTS oracle_events (
    id UUID PRIMARY KEY,
    escrow_id BIGINT NOT NULL REFERENCES escrows(escrow_id),
    oracle_address TEXT NOT NULL,
    data_type oracle_data_type NOT NULL,
    payload_hash TEXT NOT NULL,
    payload JSONB NOT NULL,
    signature TEXT NOT NULL,
    status oracle_event_status NOT NULL DEFAULT 'pending',
    tx_hash TEXT,  -- Soroban tx hash when submitted
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Prevent duplicate confirmations from same oracle for same escrow
    CONSTRAINT unique_oracle_escrow_confirmation UNIQUE (escrow_id, oracle_address)
);

-- Indexes for query performance
CREATE INDEX idx_oracle_events_escrow_id ON oracle_events(escrow_id);
CREATE INDEX idx_oracle_events_oracle_address ON oracle_events(oracle_address);
CREATE INDEX idx_oracle_events_status ON oracle_events(status);
CREATE INDEX idx_oracle_events_created_at ON oracle_events(created_at DESC);

-- Oracle audit log table for compliance tracking
CREATE TABLE IF NOT EXISTS oracle_audit_logs (
    id UUID PRIMARY KEY,
    oracle_event_id UUID REFERENCES oracle_events(id),
    action TEXT NOT NULL,  -- 'confirm', 'aggregate', 'dispute', 'resolve', 'reject'
    actor_address TEXT NOT NULL,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for audit log queries
CREATE INDEX idx_oracle_audit_logs_event_id ON oracle_audit_logs(oracle_event_id);
CREATE INDEX idx_oracle_audit_logs_action ON oracle_audit_logs(action);
CREATE INDEX idx_oracle_audit_logs_created_at ON oracle_audit_logs(created_at DESC);

-- Trigger to auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_oracle_events_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER trigger_oracle_events_updated_at 
    BEFORE UPDATE ON oracle_events
    FOR EACH ROW EXECUTE FUNCTION update_oracle_events_updated_at();
