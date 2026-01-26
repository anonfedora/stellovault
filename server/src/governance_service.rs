//! Governance service for managing proposals, votes, and protocol parameters

use crate::models::{
    GovernanceProposal, GovernanceVote, GovernanceParameter, GovernanceAuditLog,
    GovernanceMetrics, GovernanceConfig, GovernanceParameterCache,
    ProposalStatus, VoteOption, AuditActionType, AuditEntityType,
    ProposalCreationRequest, VoteSubmissionRequest
};
use sqlx::{PgPool, Error};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Governance service for managing proposals, votes, and protocol parameters
pub struct GovernanceService {
    pool: PgPool,
    parameter_cache: Arc<RwLock<GovernanceParameterCache>>,
    governance_contract_id: String,
    network_passphrase: String,
}

impl GovernanceService {
    /// Create a new governance service instance
    pub fn new(
        pool: PgPool,
        governance_contract_id: String,
        network_passphrase: String,
    ) -> Self {
        let cache = GovernanceParameterCache {
            voting_period_hours: 168, // 7 days
            execution_delay_hours: 24, // 1 day
            quorum_percentage: 0.1, // 10%
            approval_threshold_percentage: 0.5, // 50%
            min_voting_power: 100,
            emergency_quorum_percentage: 0.05, // 5%
            emergency_approval_threshold_percentage: 0.75, // 75%
            last_updated: Utc::now(),
        };

        Self {
            pool,
            parameter_cache: Arc::new(RwLock::new(cache)),
            governance_contract_id,
            network_passphrase,
        }
    }

    /// Get all governance proposals with optional filtering
    pub async fn get_proposals(
        &self,
        status: Option<ProposalStatus>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<GovernanceProposal>, Error> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let proposals = if let Some(status) = status {
            sqlx::query_as::<_, GovernanceProposal>(
                r#"
                SELECT id, proposal_id, title, description, proposer, proposal_type as "proposal_type: _",
                       status as "status: _", voting_start, voting_end, execution_time,
                       for_votes, against_votes, abstain_votes, quorum_required, approval_threshold,
                       executed_at, created_at, updated_at
                FROM governance_proposals
                WHERE status = $1::proposal_status
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#
            )
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, GovernanceProposal>(
                r#"
                SELECT id, proposal_id, title, description, proposer, proposal_type as "proposal_type: _",
                       status as "status: _", voting_start, voting_end, execution_time,
                       for_votes, against_votes, abstain_votes, quorum_required, approval_threshold,
                       executed_at, created_at, updated_at
                FROM governance_proposals
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(proposals)
    }

    /// Get a specific proposal by ID
    pub async fn get_proposal(&self, proposal_id: &str) -> Result<Option<GovernanceProposal>, Error> {
        let proposal = sqlx::query_as::<_, GovernanceProposal>(
            r#"
            SELECT id, proposal_id, title, description, proposer, proposal_type as "proposal_type: _",
                   status as "status: _", voting_start, voting_end, execution_time,
                   for_votes, against_votes, abstain_votes, quorum_required, approval_threshold,
                   executed_at, created_at, updated_at
            FROM governance_proposals
            WHERE proposal_id = $1
            "#
        )
        .bind(proposal_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(proposal)
    }

    /// Get votes for a specific proposal
    pub async fn get_proposal_votes(&self, proposal_id: &str) -> Result<Vec<GovernanceVote>, Error> {
        let votes = sqlx::query_as::<_, GovernanceVote>(
            r#"
            SELECT id, proposal_id, voter, vote_option as "vote_option: _", voting_power,
                   transaction_hash, voted_at
            FROM governance_votes
            WHERE proposal_id = $1
            ORDER BY voted_at DESC
            "#
        )
        .bind(proposal_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(votes)
    }

    /// Submit a vote on a proposal (calls Soroban contract)
    pub async fn submit_vote(&self, request: VoteSubmissionRequest) -> Result<GovernanceVote, Error> {
        // Verify proposal exists and is active
        let proposal = self.get_proposal(&request.proposal_id).await?;
        let proposal = match proposal {
            Some(p) => p,
            None => return Err(Error::RowNotFound),
        };

        if proposal.status != ProposalStatus::Active {
            return Err(Error::Protocol("Proposal is not active".to_string()));
        }

        // Check if user already voted
        let existing_vote: Option<Uuid> = sqlx::query_scalar(
            r#"SELECT id FROM governance_votes WHERE proposal_id = $1 AND voter = $2"#
        )
        .bind(&request.proposal_id)
        .bind(&request.voter_address)
        .fetch_optional(&self.pool)
        .await?;

        if existing_vote.is_some() {
            return Err(Error::Protocol("User has already voted on this proposal".to_string()));
        }

        // Get voting power (simplified - in real implementation, get from staking contract)
        let voting_power = self.get_voting_power(&request.voter_address).await?;

        // Submit vote to Soroban contract
        let transaction_hash = self.submit_vote_to_soroban(&request, voting_power).await?;

        // Record vote in database
        let vote = sqlx::query_as::<_, GovernanceVote>(
            r#"
            INSERT INTO governance_votes (proposal_id, voter, vote_option, voting_power, transaction_hash)
            VALUES ($1, $2, $3::vote_option, $4, $5)
            RETURNING id, proposal_id, voter, vote_option as "vote_option: _", voting_power,
                      transaction_hash, voted_at
            "#
        )
        .bind(&request.proposal_id)
        .bind(&request.voter_address)
        .bind(request.vote_option.clone())
        .bind(voting_power)
        .bind(&transaction_hash)
        .fetch_one(&self.pool)
        .await?;

        // Update proposal vote counts
        self.update_proposal_vote_counts(&request.proposal_id).await?;

        // Log audit event
        self.log_audit_event(
            AuditActionType::VoteCast,
            AuditEntityType::Vote,
            &vote.id.to_string(),
            &request.voter_address,
            None,
            Some(serde_json::json!({
                "proposal_id": request.proposal_id,
                "vote_option": request.vote_option,
                "voting_power": voting_power
            })),
            transaction_hash.clone(),
        ).await?;

        Ok(vote)
    }

    /// Create a new governance proposal
    pub async fn create_proposal(&self, request: ProposalCreationRequest, proposer: &str) -> Result<GovernanceProposal, Error> {
        let config = self.get_governance_config().await?;

        let voting_start = Utc::now();
        let voting_end = voting_start + Duration::hours(config.voting_period_hours as i64);
        let execution_time = request.execution_time
            .unwrap_or_else(|| voting_end + Duration::hours(config.execution_delay_hours as i64));

        // Create proposal in Soroban contract first
        let proposal_id = self.create_proposal_in_soroban(&request, proposer).await?;

        // Record proposal in database
        let proposal = sqlx::query_as::<_, GovernanceProposal>(
            r#"
            INSERT INTO governance_proposals (
                proposal_id, title, description, proposer, proposal_type,
                voting_start, voting_end, execution_time, quorum_required, approval_threshold
            )
            VALUES ($1, $2, $3, $4, $5::proposal_type, $6, $7, $8, $9, $10)
            RETURNING id, proposal_id, title, description, proposer, proposal_type as "proposal_type: _",
                      status as "status: _", voting_start, voting_end, execution_time,
                      for_votes, against_votes, abstain_votes, quorum_required, approval_threshold,
                      executed_at, created_at, updated_at
            "#
        )
        .bind(proposal_id)
        .bind(&request.title)
        .bind(&request.description)
        .bind(proposer)
        .bind(request.proposal_type.clone())
        .bind(voting_start)
        .bind(voting_end)
        .bind(execution_time)
        .bind(1000) // quorum_required - should be calculated based on total voting power
        .bind(config.approval_threshold_percentage)
        .fetch_one(&self.pool)
        .await?;

        // Log audit event
        self.log_audit_event(
            AuditActionType::ProposalCreated,
            AuditEntityType::Proposal,
            &proposal.id.to_string(),
            proposer,
            None,
            Some(serde_json::json!({
                "title": request.title,
                "description": request.description,
                "proposal_type": request.proposal_type
            })),
            None,
        ).await?;

        Ok(proposal)
    }

    /// Get governance metrics
    pub async fn get_governance_metrics(&self) -> Result<GovernanceMetrics, Error> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_proposals: Option<i64>,
            active_proposals: Option<i64>,
            total_votes: Option<i64>,
            successful_proposals: Option<i64>,
            failed_proposals: Option<i64>,
        }

        let metrics: MetricsRow = sqlx::query_as::<_, MetricsRow>(
            r#"
            SELECT
                (SELECT COUNT(*) FROM governance_proposals) as total_proposals,
                (SELECT COUNT(*) FROM governance_proposals WHERE status = 'active') as active_proposals,
                (SELECT COALESCE(SUM(for_votes + against_votes + abstain_votes), 0) FROM governance_proposals) as total_votes,
                (SELECT COUNT(*) FROM governance_proposals WHERE status = 'succeeded') as successful_proposals,
                (SELECT COUNT(*) FROM governance_proposals WHERE status = 'failed') as failed_proposals
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let participation_rate = if metrics.total_proposals.unwrap_or(0) > 0 {
            // Simplified calculation - in real implementation, calculate based on eligible voters
            (metrics.total_votes.unwrap_or(0) as f64 / (metrics.total_proposals.unwrap_or(0) * 1000) as f64).min(1.0)
        } else {
            0.0
        };

        let average_voting_time = 72.0; // Simplified - calculate actual average in real implementation

        Ok(GovernanceMetrics {
            total_proposals: metrics.total_proposals.unwrap_or(0),
            active_proposals: metrics.active_proposals.unwrap_or(0),
            total_votes: metrics.total_votes.unwrap_or(0),
            participation_rate,
            average_voting_time,
            successful_proposals: metrics.successful_proposals.unwrap_or(0),
            failed_proposals: metrics.failed_proposals.unwrap_or(0),
        })
    }

    /// Get governance configuration from cache
    pub async fn get_governance_config(&self) -> Result<GovernanceConfig, Error> {
        let cache = self.parameter_cache.read().await;
        Ok(GovernanceConfig {
            voting_period_hours: cache.voting_period_hours,
            execution_delay_hours: cache.execution_delay_hours,
            quorum_percentage: cache.quorum_percentage,
            approval_threshold_percentage: cache.approval_threshold_percentage,
            min_voting_power: cache.min_voting_power,
            emergency_quorum_percentage: cache.emergency_quorum_percentage,
            emergency_approval_threshold_percentage: cache.emergency_approval_threshold_percentage,
        })
    }

    /// Force refresh of parameter cache from database
    pub async fn refresh_parameter_cache(&self) -> Result<(), Error> {
        #[derive(sqlx::FromRow)]
        struct ParameterRow {
            parameter_key: String,
            parameter_value: serde_json::Value,
        }

        let parameters: Vec<ParameterRow> = sqlx::query_as::<_, ParameterRow>(
            r#"
            SELECT parameter_key, parameter_value
            FROM governance_parameters
            WHERE is_active = true AND effective_from <= NOW()
            AND (effective_until IS NULL OR effective_until > NOW())
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut cache = self.parameter_cache.write().await;

        for param in parameters {
            match param.parameter_key.as_str() {
                "voting_period_hours" => {
                    if let Some(value) = param.parameter_value.as_i64() {
                        cache.voting_period_hours = value as i32;
                    }
                }
                "execution_delay_hours" => {
                    if let Some(value) = param.parameter_value.as_i64() {
                        cache.execution_delay_hours = value as i32;
                    }
                }
                "quorum_percentage" => {
                    if let Some(value) = param.parameter_value.as_f64() {
                        cache.quorum_percentage = value;
                    }
                }
                "approval_threshold_percentage" => {
                    if let Some(value) = param.parameter_value.as_f64() {
                        cache.approval_threshold_percentage = value;
                    }
                }
                "min_voting_power" => {
                    if let Some(value) = param.parameter_value.as_i64() {
                        cache.min_voting_power = value;
                    }
                }
                "emergency_quorum_percentage" => {
                    if let Some(value) = param.parameter_value.as_f64() {
                        cache.emergency_quorum_percentage = value;
                    }
                }
                "emergency_approval_threshold_percentage" => {
                    if let Some(value) = param.parameter_value.as_f64() {
                        cache.emergency_approval_threshold_percentage = value;
                    }
                }
                _ => {}
            }
        }

        cache.last_updated = Utc::now();
        Ok(())
    }

    /// Check if an action is allowed based on governance parameters
    pub async fn check_governance_enforcement(&self, action: &str, parameters: serde_json::Value) -> Result<bool, Error> {
        // This is a simplified implementation - in reality, this would check various
        // governance parameters and potentially call Soroban contracts
        let config = self.get_governance_config().await?;

        match action {
            "escrow_creation" => {
                // Check if escrow creation is allowed based on governance
                Ok(true) // Simplified - would check governance flags
            }
            "oracle_registration" => {
                // Check oracle registration requirements
                Ok(true) // Simplified
            }
            "parameter_change" => {
                // Validate parameter change against governance rules
                Ok(true) // Simplified
            }
            _ => Ok(true),
        }
    }

    // Helper methods

    async fn get_voting_power(&self, voter_address: &str) -> Result<i64, Error> {
        // Simplified - in real implementation, query staking contract or token balance
        // For now, return a default voting power
        Ok(100)
    }

    async fn submit_vote_to_soroban(&self, request: &VoteSubmissionRequest, voting_power: i64) -> Result<Option<String>, Error> {
        // TODO: Implement actual Soroban contract call
        // For now, simulate transaction hash
        Ok(Some(format!("tx_{}", Uuid::new_v4().simple())))
    }

    async fn create_proposal_in_soroban(&self, request: &ProposalCreationRequest, proposer: &str) -> Result<String, Error> {
        // TODO: Implement actual Soroban contract call
        // For now, generate a proposal ID
        Ok(format!("proposal_{}", Uuid::new_v4().simple()))
    }

    async fn update_proposal_vote_counts(&self, proposal_id: &str) -> Result<(), Error> {
        sqlx::query(
            r#"
            UPDATE governance_proposals
            SET
                for_votes = (SELECT COALESCE(SUM(voting_power), 0) FROM governance_votes WHERE proposal_id = $1 AND vote_option = 'for'),
                against_votes = (SELECT COALESCE(SUM(voting_power), 0) FROM governance_votes WHERE proposal_id = $1 AND vote_option = 'against'),
                abstain_votes = (SELECT COALESCE(SUM(voting_power), 0) FROM governance_votes WHERE proposal_id = $1 AND vote_option = 'abstain'),
                updated_at = NOW()
            WHERE proposal_id = $1
            "#
        )
        .bind(proposal_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn log_audit_event(
        &self,
        action_type: AuditActionType,
        entity_type: AuditEntityType,
        entity_id: &str,
        user_address: &str,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        transaction_hash: Option<String>,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO governance_audit_log (
                action_type, entity_type, entity_id, user_address,
                old_value, new_value, transaction_hash
            )
            VALUES ($1::audit_action_type, $2::audit_entity_type, $3, $4, $5, $6, $7)
            "#
        )
        .bind(action_type as AuditActionType)
        .bind(entity_type as AuditEntityType)
        .bind(entity_id)
        .bind(user_address)
        .bind(old_value)
        .bind(new_value)
        .bind(transaction_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}