-- Migration to add KYC/AML compliance fields to users table

-- Create KYC status enum
DO $$ BEGIN
    CREATE TYPE kyc_status AS ENUM ('unverified', 'pending', 'verified', 'rejected', 'expired');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Add KYC fields to users table
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS kyc_status kyc_status NOT NULL DEFAULT 'unverified',
ADD COLUMN IF NOT EXISTS kyc_expiry TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS kyc_provider TEXT,
ADD COLUMN IF NOT EXISTS kyc_verified_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS kyc_reference_id TEXT;

-- Create index for KYC status queries
CREATE INDEX IF NOT EXISTS idx_users_kyc_status ON users(kyc_status);
CREATE INDEX IF NOT EXISTS idx_users_kyc_expiry ON users(kyc_expiry) WHERE kyc_status = 'verified';

-- Function to check if user KYC is expired
CREATE OR REPLACE FUNCTION check_kyc_expiry()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.kyc_status = 'verified' AND NEW.kyc_expiry IS NOT NULL AND NEW.kyc_expiry < NOW() THEN
        NEW.kyc_status = 'expired';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically expire KYC on update
DROP TRIGGER IF EXISTS trigger_check_kyc_expiry ON users;
CREATE TRIGGER trigger_check_kyc_expiry
    BEFORE UPDATE ON users
    FOR EACH ROW
    WHEN (OLD.kyc_status IS DISTINCT FROM NEW.kyc_status OR OLD.kyc_expiry IS DISTINCT FROM NEW.kyc_expiry)
    EXECUTE FUNCTION check_kyc_expiry();

-- Function to bulk expire KYC (can be called periodically via cron)
CREATE OR REPLACE FUNCTION expire_kyc_statuses()
RETURNS INTEGER AS $$
DECLARE
    updated_count INTEGER;
BEGIN
    UPDATE users 
    SET kyc_status = 'expired'
    WHERE kyc_status = 'verified' 
      AND kyc_expiry IS NOT NULL 
      AND kyc_expiry < NOW();
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RETURN updated_count;
END;
$$ LANGUAGE plpgsql;
