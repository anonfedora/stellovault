-- Migration for escrow tables
-- Create escrow status enum
CREATE TYPE escrow_status AS ENUM ('pending', 'active', 'released', 'cancelled', 'timedout', 'disputed');

-- Create escrows table
CREATE TABLE IF NOT EXISTS escrows (
    id UUID PRIMARY KEY,
    escrow_id BIGINT NOT NULL UNIQUE,
    buyer_id UUID NOT NULL,
    seller_id UUID NOT NULL,
    collateral_id UUID NOT NULL,
    amount BIGINT NOT NULL,
    status escrow_status NOT NULL DEFAULT 'pending',
    oracle_address TEXT NOT NULL,
    release_conditions TEXT NOT NULL,
    timeout_at TIMESTAMPTZ,
    disputed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys (assuming users and collateral_tokens tables exist)
    CONSTRAINT fk_buyer FOREIGN KEY (buyer_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_seller FOREIGN KEY (seller_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_collateral FOREIGN KEY (collateral_id) REFERENCES collateral_tokens(id) ON DELETE CASCADE,
    
    -- Constraints
    CONSTRAINT check_amount_positive CHECK (amount > 0),
    CONSTRAINT check_different_parties CHECK (buyer_id != seller_id)
);

-- Create indexes for better query performance
CREATE INDEX idx_escrows_status ON escrows(status);
CREATE INDEX idx_escrows_buyer_id ON escrows(buyer_id);
CREATE INDEX idx_escrows_seller_id ON escrows(seller_id);
CREATE INDEX idx_escrows_timeout_at ON escrows(timeout_at) WHERE timeout_at IS NOT NULL;
CREATE INDEX idx_escrows_created_at ON escrows(created_at DESC);
CREATE INDEX idx_escrows_escrow_id ON escrows(escrow_id);

-- Create trigger to auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_escrows_updated_at BEFORE UPDATE ON escrows
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
