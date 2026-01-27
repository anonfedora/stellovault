-- Create asset type enum (if not exists)
DO $$ BEGIN
    CREATE TYPE asset_type AS ENUM ('INVOICE', 'COMMODITY', 'RECEIVABLE');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create token status enum (if not exists)
DO $$ BEGIN
    CREATE TYPE token_status AS ENUM ('active', 'locked', 'burned');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create collateral table
CREATE TABLE IF NOT EXISTS collateral (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id TEXT NOT NULL UNIQUE, -- Soroban contract token ID
    owner_id UUID NOT NULL,
    asset_type asset_type NOT NULL,
    asset_value BIGINT NOT NULL,
    metadata_hash TEXT NOT NULL UNIQUE,
    fractional_shares INTEGER NOT NULL DEFAULT 1,
    status token_status NOT NULL DEFAULT 'active',
    tx_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign keys
    -- Assuming users table exists from migration 001
    CONSTRAINT fk_collateral_owner FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_collateral_owner_id ON collateral(owner_id);
CREATE INDEX IF NOT EXISTS idx_collateral_status ON collateral(status);
CREATE INDEX IF NOT EXISTS idx_collateral_token_id ON collateral(token_id);
CREATE INDEX IF NOT EXISTS idx_collateral_metadata_hash ON collateral(metadata_hash);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

DROP TRIGGER IF EXISTS update_collateral_updated_at ON collateral;
CREATE TRIGGER update_collateral_updated_at BEFORE UPDATE ON collateral
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
