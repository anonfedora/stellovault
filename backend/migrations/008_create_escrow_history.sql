-- Migration: escrow audit/history table
CREATE TABLE IF NOT EXISTS escrow_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    escrow_id UUID NOT NULL REFERENCES escrows(id) ON DELETE CASCADE,
    action TEXT NOT NULL,
    old_status escrow_status,
    new_status escrow_status,
    actor_id UUID,
    metadata JSONB,
    tx_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_escrow_history_escrow_id ON escrow_history(escrow_id);
CREATE INDEX IF NOT EXISTS idx_escrow_history_created_at ON escrow_history(created_at DESC);
