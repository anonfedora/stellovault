-- Migration for collateral registry table
-- Create collateral status enum
CREATE TYPE collateral_status AS ENUM ('active', 'locked', 'expired', 'burned');

-- Create collateral table
CREATE TABLE IF NOT EXISTS collateral (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    collateral_id BIGINT NOT NULL UNIQUE, -- Soroban contract collateral ID
    owner_id UUID NOT NULL,
    face_value BIGINT NOT NULL,
    expiry_ts BIGINT NOT NULL,
    metadata_hash TEXT NOT NULL UNIQUE, -- Prevents double-collateralization
    registered_at TIMESTAMPTZ NOT NULL,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    status collateral_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign keys
    CONSTRAINT fk_collateral_owner FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE,

    -- Constraints
    CONSTRAINT check_face_value_positive CHECK (face_value > 0),
    CONSTRAINT check_expiry_future CHECK (expiry_ts > extract(epoch from now())),
    CONSTRAINT unique_metadata_hash UNIQUE (metadata_hash)
);

-- Create indexes for better query performance
CREATE INDEX idx_collateral_status ON collateral(status);
CREATE INDEX idx_collateral_owner_id ON collateral(owner_id);
CREATE INDEX idx_collateral_locked ON collateral(locked);
CREATE INDEX idx_collateral_expiry_ts ON collateral(expiry_ts);
CREATE INDEX idx_collateral_metadata_hash ON collateral(metadata_hash);
CREATE INDEX idx_collateral_registered_at ON collateral(registered_at DESC);

-- Create trigger to auto-update updated_at timestamp
CREATE TRIGGER update_collateral_updated_at BEFORE UPDATE ON collateral
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();