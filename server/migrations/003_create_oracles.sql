-- Create oracle registry table
CREATE TABLE oracles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address VARCHAR(56) UNIQUE NOT NULL,
    name VARCHAR(255),
    endpoint_url TEXT,
    public_key TEXT,
    is_active BOOLEAN DEFAULT true,
    reputation_score DECIMAL(5,2) DEFAULT 100.00,
    total_confirmations INTEGER DEFAULT 0,
    successful_confirmations INTEGER DEFAULT 0,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    added_by UUID,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT valid_reputation_score CHECK (reputation_score >= 0 AND reputation_score <= 100),
    CONSTRAINT valid_address_format CHECK (length(address) >= 32)
);

-- Create oracle confirmations table
CREATE TABLE oracle_confirmations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    escrow_id VARCHAR(56) NOT NULL,
    oracle_address VARCHAR(56) NOT NULL,
    event_type INTEGER NOT NULL,
    result JSONB,
    signature TEXT NOT NULL,
    transaction_hash VARCHAR(64),
    block_number BIGINT,
    gas_used BIGINT,
    confirmed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    verification_status VARCHAR(20) DEFAULT 'pending',
    error_message TEXT,

    FOREIGN KEY (oracle_address) REFERENCES oracles(address),
    CONSTRAINT valid_event_type CHECK (event_type IN (1, 2, 3, 4)),
    CONSTRAINT valid_verification_status CHECK (verification_status IN ('pending', 'verified', 'failed')),
    CONSTRAINT unique_oracle_escrow_confirmation UNIQUE (escrow_id, oracle_address)
);

-- Create indexes for performance
CREATE INDEX idx_oracles_address ON oracles(address);
CREATE INDEX idx_oracles_active ON oracles(is_active);
CREATE INDEX idx_oracle_confirmations_escrow ON oracle_confirmations(escrow_id);
CREATE INDEX idx_oracle_confirmations_oracle ON oracle_confirmations(oracle_address);
CREATE INDEX idx_oracle_confirmations_status ON oracle_confirmations(verification_status);
CREATE INDEX idx_oracle_confirmations_timestamp ON oracle_confirmations(confirmed_at);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add trigger to oracles table
CREATE TRIGGER update_oracles_updated_at
    BEFORE UPDATE ON oracles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE oracles IS 'Registry of trusted oracle providers for off-chain event verification';
COMMENT ON TABLE oracle_confirmations IS 'Storage of oracle confirmations for escrow events';
COMMENT ON COLUMN oracles.reputation_score IS 'Oracle reliability score (0-100, higher is better)';
COMMENT ON COLUMN oracle_confirmations.event_type IS 'Event type: 1=Shipment, 2=Delivery, 3=Quality, 4=Custom';
COMMENT ON COLUMN oracle_confirmations.verification_status IS 'Status of signature verification: pending, verified, failed';