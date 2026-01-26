-- Create asset type enum
CREATE TYPE asset_type AS ENUM ('INVOICE', 'COMMODITY', 'RECEIVABLE');

-- Create token status enum
CREATE TYPE token_status AS ENUM ('active', 'locked', 'burned');

-- Create collateral_tokens table
CREATE TABLE IF NOT EXISTS collateral_tokens (
    id UUID PRIMARY KEY,
    token_id TEXT NOT NULL UNIQUE,
    owner_id UUID NOT NULL, -- References users(id)
    asset_type asset_type NOT NULL,
    asset_value BIGINT NOT NULL,
    metadata_hash TEXT NOT NULL,
    fractional_shares INTEGER NOT NULL DEFAULT 1,
    status token_status NOT NULL DEFAULT 'active',
    tx_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add foreign key if users table exists, otherwise we assume it will be handled
-- Note: In a real scenario we'd ensure users table exists. 
-- Adding constraint if users table is expected to be there.
-- DO NOT ADD CONSTRAINT if users table creation is missing in previous migrations to avoid failure.
-- However, based on 001, users table is expected.
-- We will attempt to add the constraint.

DO $$
BEGIN
    IF EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'users') THEN
        ALTER TABLE collateral_tokens 
        ADD CONSTRAINT fk_collateral_owner FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE;
    END IF;
END $$;

-- Indexes
CREATE INDEX idx_collateral_owner_id ON collateral_tokens(owner_id);
CREATE INDEX idx_collateral_status ON collateral_tokens(status);
CREATE INDEX idx_collateral_token_id ON collateral_tokens(token_id);

-- Trigger for updated_at
CREATE TRIGGER update_collateral_updated_at BEFORE UPDATE ON collateral_tokens
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
