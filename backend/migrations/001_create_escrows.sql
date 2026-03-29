-- Migration for escrow tables
-- Create escrow status enum
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'escrow_status') THEN
        CREATE TYPE escrow_status AS ENUM ('pending', 'active', 'released', 'cancelled', 'timedout', 'disputed');
    END IF;
END $$;

-- Create escrows table
CREATE TABLE IF NOT EXISTS escrows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    escrow_id BIGINT NOT NULL UNIQUE,
    buyer_id UUID NOT NULL,
    seller_id UUID NOT NULL,
    collateral_id TEXT NOT NULL,
    amount BIGINT NOT NULL,
    status escrow_status NOT NULL DEFAULT 'pending',
    oracle_address TEXT NOT NULL,
    release_conditions TEXT NOT NULL,
    timeout_at TIMESTAMPTZ,
    disputed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_buyer FOREIGN KEY (buyer_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_seller FOREIGN KEY (seller_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT check_amount_positive CHECK (amount > 0),
    CONSTRAINT check_different_parties CHECK (buyer_id != seller_id)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_escrows_status ON escrows(status);
CREATE INDEX IF NOT EXISTS idx_escrows_buyer_id ON escrows(buyer_id);
CREATE INDEX IF NOT EXISTS idx_escrows_seller_id ON escrows(seller_id);
CREATE INDEX IF NOT EXISTS idx_escrows_timeout_at ON escrows(timeout_at) WHERE timeout_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_escrows_created_at ON escrows(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_escrows_escrow_id ON escrows(escrow_id);

-- Create trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_escrows_updated_at BEFORE UPDATE ON escrows
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
