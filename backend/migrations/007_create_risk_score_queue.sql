-- Migration 007: Risk Score Update Queue
-- Creates a queue table for async risk score updates

CREATE TABLE IF NOT EXISTS risk_score_update_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trigger_type VARCHAR(50) NOT NULL, -- 'repayment', 'manual', 'scheduled'
    processed_at TIMESTAMPTZ,
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Partial unique index for pending updates
CREATE UNIQUE INDEX IF NOT EXISTS idx_risk_score_queue_unique_pending 
    ON risk_score_update_queue(user_id, trigger_type) 
    WHERE processed_at IS NULL;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_risk_score_queue_user_id ON risk_score_update_queue(user_id);
CREATE INDEX IF NOT EXISTS idx_risk_score_queue_pending ON risk_score_update_queue(created_at) 
    WHERE processed_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_risk_score_queue_processed ON risk_score_update_queue(processed_at) 
    WHERE processed_at IS NOT NULL;

-- Trigger for updated_at
DROP TRIGGER IF EXISTS update_risk_score_queue_updated_at ON risk_score_update_queue;
CREATE TRIGGER update_risk_score_queue_updated_at
    BEFORE UPDATE ON risk_score_update_queue
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to process pending risk score updates
CREATE OR REPLACE FUNCTION get_pending_risk_score_updates(batch_size INTEGER DEFAULT 10)
RETURNS TABLE (
    id UUID,
    user_id UUID,
    trigger_type VARCHAR(50),
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        q.id,
        q.user_id,
        q.trigger_type,
        q.created_at
    FROM risk_score_update_queue q
    WHERE q.processed_at IS NULL
    AND q.retry_count < 3
    ORDER BY q.created_at ASC
    LIMIT batch_size
    FOR UPDATE SKIP LOCKED;
END;
$$ LANGUAGE plpgsql;

-- Function to mark update as processed
CREATE OR REPLACE FUNCTION mark_risk_score_update_processed(
    update_id UUID,
    success BOOLEAN,
    error_msg TEXT DEFAULT NULL
)
RETURNS VOID AS $$
BEGIN
    IF success THEN
        UPDATE risk_score_update_queue
        SET processed_at = NOW(),
            updated_at = NOW()
        WHERE id = update_id;
    ELSE
        UPDATE risk_score_update_queue
        SET retry_count = retry_count + 1,
            error_message = error_msg,
            updated_at = NOW()
        WHERE id = update_id;
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON TABLE risk_score_update_queue IS 'Queue for async risk score recalculation triggered by events';
COMMENT ON FUNCTION get_pending_risk_score_updates IS 'Fetch pending risk score updates for background processing';
COMMENT ON FUNCTION mark_risk_score_update_processed IS 'Mark a risk score update as processed or failed';
