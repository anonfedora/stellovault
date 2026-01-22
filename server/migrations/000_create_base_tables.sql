-- Create user roles enum
DO $$ BEGIN
    CREATE TYPE user_role AS ENUM ('buyer', 'seller', 'oracle', 'admin');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    stellar_address TEXT NOT NULL UNIQUE,
    email TEXT,
    name TEXT,
    role user_role NOT NULL DEFAULT 'buyer',
    risk_score INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create asset types enum
DO $$ BEGIN
    CREATE TYPE asset_type AS ENUM ('INVOICE', 'COMMODITY', 'RECEIVABLE');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create token status enum
DO $$ BEGIN
    CREATE TYPE token_status AS ENUM ('active', 'locked', 'burned');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create collateral_tokens table
CREATE TABLE IF NOT EXISTS collateral_tokens (
    id UUID PRIMARY KEY,
    token_id TEXT NOT NULL UNIQUE,
    owner_id UUID NOT NULL,
    asset_type asset_type NOT NULL,
    asset_value BIGINT NOT NULL,
    metadata_hash TEXT NOT NULL,
    fractional_shares INTEGER NOT NULL DEFAULT 1,
    status token_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_owner FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT check_asset_value_positive CHECK (asset_value > 0),
    CONSTRAINT check_fractional_shares_positive CHECK (fractional_shares > 0)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_collateral_owner_id ON collateral_tokens(owner_id);
CREATE INDEX IF NOT EXISTS idx_collateral_status ON collateral_tokens(status);
CREATE INDEX IF NOT EXISTS idx_collateral_token_id ON collateral_tokens(token_id);
