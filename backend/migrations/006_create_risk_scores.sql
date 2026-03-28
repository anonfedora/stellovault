-- Migration 006: Risk Scores and Components
-- Creates tables for granular risk scoring system

-- =============================================================================
-- Risk Tier Enum
-- =============================================================================
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'risk_tier') THEN
        CREATE TYPE risk_tier AS ENUM ('excellent', 'good', 'fair', 'poor', 'very_poor', 'unscored');
    END IF;
END $$;

-- =============================================================================
-- Main Risk Scores Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS risk_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Overall score (0-1000 scale)
    overall_score INTEGER NOT NULL CHECK (overall_score >= 0 AND overall_score <= 1000),
    risk_tier risk_tier NOT NULL DEFAULT 'unscored',
    confidence FLOAT NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    
    -- Component scores (0-1000 scale)
    on_chain_activity_score INTEGER NOT NULL DEFAULT 0 CHECK (on_chain_activity_score >= 0 AND on_chain_activity_score <= 1000),
    repayment_history_score INTEGER NOT NULL DEFAULT 0 CHECK (repayment_history_score >= 0 AND repayment_history_score <= 1000),
    collateral_quality_score INTEGER NOT NULL DEFAULT 0 CHECK (collateral_quality_score >= 0 AND collateral_quality_score <= 1000),
    document_verification_score INTEGER CHECK (document_verification_score >= 0 AND document_verification_score <= 100),
    
    -- Component weights (should sum to 1.0)
    on_chain_activity_weight FLOAT NOT NULL DEFAULT 0.40 CHECK (on_chain_activity_weight >= 0 AND on_chain_activity_weight <= 1),
    repayment_history_weight FLOAT NOT NULL DEFAULT 0.40 CHECK (repayment_history_weight >= 0 AND repayment_history_weight <= 1),
    collateral_quality_weight FLOAT NOT NULL DEFAULT 0.20 CHECK (collateral_quality_weight >= 0 AND collateral_quality_weight <= 1),
    
    -- Metadata
    calculation_version VARCHAR(10) NOT NULL DEFAULT 'v2.0',
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure weights sum to 1.0 (with small tolerance for floating point)
    CONSTRAINT weights_sum_check CHECK (
        ABS((on_chain_activity_weight + repayment_history_weight + collateral_quality_weight) - 1.0) < 0.001
    )
);

-- =============================================================================
-- DeFi Activity Metrics Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS defi_activity_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    risk_score_id UUID NOT NULL REFERENCES risk_scores(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    wallet_address VARCHAR(56) NOT NULL, -- Stellar address (G...)
    
    -- Soroswap metrics
    soroswap_tx_count INTEGER NOT NULL DEFAULT 0,
    soroswap_volume_usd BIGINT NOT NULL DEFAULT 0,
    soroswap_liquidity_provided BIGINT NOT NULL DEFAULT 0,
    soroswap_last_activity_at TIMESTAMPTZ,
    
    -- Aquarius metrics
    aquarius_tx_count INTEGER NOT NULL DEFAULT 0,
    aquarius_volume_usd BIGINT NOT NULL DEFAULT 0,
    aquarius_positions_count INTEGER NOT NULL DEFAULT 0,
    aquarius_last_activity_at TIMESTAMPTZ,
    
    -- Aggregated metrics
    total_defi_tx_count INTEGER NOT NULL DEFAULT 0,
    total_defi_volume_usd BIGINT NOT NULL DEFAULT 0,
    protocol_diversity_score INTEGER NOT NULL DEFAULT 0 CHECK (protocol_diversity_score >= 0 AND protocol_diversity_score <= 100),
    activity_consistency_score INTEGER NOT NULL DEFAULT 0 CHECK (activity_consistency_score >= 0 AND activity_consistency_score <= 100),
    
    -- Time range for metrics
    metrics_start_date TIMESTAMPTZ NOT NULL,
    metrics_end_date TIMESTAMPTZ NOT NULL,
    
    -- Metadata
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- Document Verification Status Enum
-- =============================================================================
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'verification_status') THEN
        CREATE TYPE verification_status AS ENUM ('pending', 'verified', 'rejected', 'manual_review', 'expired');
    END IF;
END $$;

-- =============================================================================
-- Document Verifications Table
-- =============================================================================
CREATE TABLE IF NOT EXISTS document_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    risk_score_id UUID REFERENCES risk_scores(id) ON DELETE SET NULL,
    
    -- Document metadata
    document_type VARCHAR(50) NOT NULL, -- 'invoice', 'contract', 'receipt', 'tax_document'
    document_url TEXT NOT NULL,
    document_hash VARCHAR(64), -- SHA256 hash for integrity
    file_size_bytes BIGINT,
    mime_type VARCHAR(100),
    
    -- OCR results
    extracted_text TEXT,
    extracted_fields JSONB, -- Structured data: {invoice_number, date, amount, vendor, etc}
    
    -- Verification results
    verification_status verification_status NOT NULL DEFAULT 'pending',
    verification_score INTEGER CHECK (verification_score >= 0 AND verification_score <= 100),
    confidence FLOAT CHECK (confidence >= 0 AND confidence <= 1),
    fraud_indicators JSONB, -- Array of detected fraud patterns
    
    -- Processing metadata
    ocr_provider VARCHAR(50), -- 'textract', 'gpt4_vision', 'tesseract'
    processing_time_ms INTEGER,
    error_message TEXT,
    
    -- Timestamps
    verified_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- Indices for Performance
-- =============================================================================

-- Risk scores indices
CREATE INDEX IF NOT EXISTS idx_risk_scores_user_id ON risk_scores(user_id);
CREATE INDEX IF NOT EXISTS idx_risk_scores_calculated_at ON risk_scores(calculated_at DESC);
CREATE INDEX IF NOT EXISTS idx_risk_scores_risk_tier ON risk_scores(risk_tier);
CREATE INDEX IF NOT EXISTS idx_risk_scores_expires_at ON risk_scores(expires_at) WHERE expires_at IS NOT NULL;

-- DeFi activity metrics indices
CREATE INDEX IF NOT EXISTS idx_defi_activity_risk_score_id ON defi_activity_metrics(risk_score_id);
CREATE INDEX IF NOT EXISTS idx_defi_activity_user_id ON defi_activity_metrics(user_id);
CREATE INDEX IF NOT EXISTS idx_defi_activity_wallet ON defi_activity_metrics(wallet_address);
CREATE INDEX IF NOT EXISTS idx_defi_activity_fetched_at ON defi_activity_metrics(fetched_at DESC);

-- Document verifications indices
CREATE INDEX IF NOT EXISTS idx_document_verifications_user_id ON document_verifications(user_id);
CREATE INDEX IF NOT EXISTS idx_document_verifications_risk_score_id ON document_verifications(risk_score_id);
CREATE INDEX IF NOT EXISTS idx_document_verifications_status ON document_verifications(verification_status);
CREATE INDEX IF NOT EXISTS idx_document_verifications_created_at ON document_verifications(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_document_verifications_document_hash ON document_verifications(document_hash) WHERE document_hash IS NOT NULL;

-- =============================================================================
-- Trigger: Update updated_at timestamp
-- =============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to risk_scores
DROP TRIGGER IF EXISTS update_risk_scores_updated_at ON risk_scores;
CREATE TRIGGER update_risk_scores_updated_at
    BEFORE UPDATE ON risk_scores
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Apply trigger to defi_activity_metrics
DROP TRIGGER IF EXISTS update_defi_activity_metrics_updated_at ON defi_activity_metrics;
CREATE TRIGGER update_defi_activity_metrics_updated_at
    BEFORE UPDATE ON defi_activity_metrics
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Apply trigger to document_verifications
DROP TRIGGER IF EXISTS update_document_verifications_updated_at ON document_verifications;
CREATE TRIGGER update_document_verifications_updated_at
    BEFORE UPDATE ON document_verifications
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- Comments for Documentation
-- =============================================================================

COMMENT ON TABLE risk_scores IS 'Stores granular risk scores with component breakdown and weights';
COMMENT ON TABLE defi_activity_metrics IS 'DeFi protocol activity metrics from Soroswap and Aquarius';
COMMENT ON TABLE document_verifications IS 'Document verification records with OCR and fraud detection results';

COMMENT ON COLUMN risk_scores.overall_score IS 'Weighted overall risk score (0-1000 scale)';
COMMENT ON COLUMN risk_scores.confidence IS 'Confidence level of the score (0-1), based on data availability';
COMMENT ON COLUMN defi_activity_metrics.protocol_diversity_score IS 'Score based on number of different protocols used (0-100)';
COMMENT ON COLUMN defi_activity_metrics.activity_consistency_score IS 'Score based on regular activity patterns (0-100)';
COMMENT ON COLUMN document_verifications.fraud_indicators IS 'JSON array of detected fraud patterns and anomalies';
