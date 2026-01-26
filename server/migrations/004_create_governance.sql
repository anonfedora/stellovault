-- Create proposal types enum
CREATE TYPE proposal_type AS ENUM ('parameter_change', 'contract_upgrade', 'treasury_action', 'emergency_action', 'custom');

-- Create proposal status enum
CREATE TYPE proposal_status AS ENUM ('pending', 'active', 'succeeded', 'failed', 'executed', 'cancelled');

-- Create vote option enum
CREATE TYPE vote_option AS ENUM ('for', 'against', 'abstain');

-- Create parameter type enum
CREATE TYPE parameter_type AS ENUM ('integer', 'float', 'boolean', 'string', 'json');

-- Create audit action type enum
CREATE TYPE audit_action_type AS ENUM ('proposal_created', 'vote_cast', 'proposal_executed', 'parameter_changed', 'emergency_action');

-- Create audit entity type enum
CREATE TYPE audit_entity_type AS ENUM ('proposal', 'vote', 'parameter', 'contract');

-- Create governance proposals table
CREATE TABLE governance_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id VARCHAR(56) UNIQUE NOT NULL, -- Soroban contract proposal ID
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    proposer VARCHAR(56) NOT NULL, -- Stellar address
    proposal_type proposal_type NOT NULL,
    status proposal_status DEFAULT 'pending',
    voting_start TIMESTAMP WITH TIME ZONE NOT NULL,
    voting_end TIMESTAMP WITH TIME ZONE NOT NULL,
    execution_time TIMESTAMP WITH TIME ZONE,
    for_votes BIGINT DEFAULT 0,
    against_votes BIGINT DEFAULT 0,
    abstain_votes BIGINT DEFAULT 0,
    quorum_required BIGINT NOT NULL,
    approval_threshold DECIMAL(5,4) NOT NULL, -- 0.0000 to 1.0000
    executed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT valid_approval_threshold CHECK (approval_threshold >= 0 AND approval_threshold <= 1),
    CONSTRAINT valid_voting_period CHECK (voting_end > voting_start),
    CONSTRAINT valid_execution_time CHECK (execution_time IS NULL OR execution_time > voting_end)
);

-- Create governance votes table
CREATE TABLE governance_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id VARCHAR(56) NOT NULL,
    voter VARCHAR(56) NOT NULL, -- Stellar address
    vote_option vote_option NOT NULL,
    voting_power BIGINT NOT NULL,
    transaction_hash VARCHAR(64),
    voted_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id),
    CONSTRAINT unique_vote_per_proposal UNIQUE (proposal_id, voter)
);

-- Create governance parameters table
CREATE TABLE governance_parameters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parameter_key VARCHAR(100) UNIQUE NOT NULL,
    parameter_value JSONB NOT NULL,
    parameter_type parameter_type NOT NULL,
    description TEXT,
    proposed_by VARCHAR(56),
    proposal_id VARCHAR(56),
    effective_from TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    effective_until TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id),
    CONSTRAINT valid_effective_period CHECK (effective_until IS NULL OR effective_until > effective_from)
);

-- Create governance audit log table
CREATE TABLE governance_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_type audit_action_type NOT NULL,
    entity_type audit_entity_type NOT NULL,
    entity_id VARCHAR(100) NOT NULL,
    user_address VARCHAR(56) NOT NULL,
    old_value JSONB,
    new_value JSONB,
    transaction_hash VARCHAR(64),
    block_number BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_governance_proposals_status ON governance_proposals(status);
CREATE INDEX idx_governance_proposals_voting_end ON governance_proposals(voting_end);
CREATE INDEX idx_governance_proposals_proposer ON governance_proposals(proposer);
CREATE INDEX idx_governance_votes_proposal ON governance_votes(proposal_id);
CREATE INDEX idx_governance_votes_voter ON governance_votes(voter);
CREATE INDEX idx_governance_parameters_key ON governance_parameters(parameter_key);
CREATE INDEX idx_governance_parameters_active ON governance_parameters(is_active);
CREATE INDEX idx_governance_audit_entity ON governance_audit_log(entity_type, entity_id);
CREATE INDEX idx_governance_audit_timestamp ON governance_audit_log(created_at);

-- Create updated_at triggers
CREATE OR REPLACE FUNCTION update_governance_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_governance_proposals_updated_at
    BEFORE UPDATE ON governance_proposals
    FOR EACH ROW
    EXECUTE FUNCTION update_governance_updated_at_column();

CREATE TRIGGER update_governance_parameters_updated_at
    BEFORE UPDATE ON governance_parameters
    FOR EACH ROW
    EXECUTE FUNCTION update_governance_updated_at_column();

-- Insert default governance parameters
INSERT INTO governance_parameters (parameter_key, parameter_value, parameter_type, description) VALUES
('voting_period_hours', '168', 'integer', 'Default voting period in hours (7 days)'),
('execution_delay_hours', '24', 'integer', 'Delay before proposal execution in hours'),
('quorum_percentage', '0.1', 'float', 'Minimum quorum percentage for proposal validity'),
('approval_threshold_percentage', '0.5', 'float', 'Minimum approval percentage for proposal success'),
('min_voting_power', '100', 'integer', 'Minimum voting power required to participate'),
('emergency_quorum_percentage', '0.05', 'float', 'Reduced quorum for emergency proposals'),
('emergency_approval_threshold_percentage', '0.75', 'float', 'Higher threshold for emergency proposals');

-- Add comments for documentation
COMMENT ON TABLE governance_proposals IS 'Governance proposals submitted by community members';
COMMENT ON TABLE governance_votes IS 'Individual votes cast on governance proposals';
COMMENT ON TABLE governance_parameters IS 'Governance protocol parameters controlled by proposals';
COMMENT ON TABLE governance_audit_log IS 'Audit trail for all governance actions';
COMMENT ON COLUMN governance_proposals.approval_threshold IS 'Percentage of for votes required (0.0-1.0)';
COMMENT ON COLUMN governance_proposals.quorum_required IS 'Minimum total votes required for validity';