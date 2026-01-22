-- Add pending and failed statuses to token_status enum
-- Note: PostgreSQL does not support IF NOT EXISTS for ADD VALUE inside a transaction block in some versions, 
-- but we are running these as simple statements.
ALTER TYPE token_status ADD VALUE IF NOT EXISTS 'pending';
ALTER TYPE token_status ADD VALUE IF NOT EXISTS 'failed';
