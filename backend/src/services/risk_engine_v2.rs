//! Enhanced Risk Engine V2 - Integrates DeFi history and document verification

use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::defi_history::{DeFiHistoryProvider, DeFiHistoryQuery};
use crate::document_verification::DocumentVerificationService;
use crate::error::ApiError;

/// Enhanced risk score response with granular components
#[derive(Debug, serde::Serialize)]
pub struct RiskScoreResponseV2 {
    pub user_id: Uuid,
    pub wallet_address: String,
    pub overall_score: i32,
    pub risk_tier: String,
    pub confidence: f64,
    pub components: ScoreComponents,
    pub calculated_at: chrono::DateTime<Utc>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

/// Individual score components with weights
#[derive(Debug, serde::Serialize)]
pub struct ScoreComponents {
    pub on_chain_activity: ComponentScore,
    pub repayment_history: ComponentScore,
    pub collateral_quality: ComponentScore,
    pub document_verification: Option<DocumentScore>,
}

#[derive(Debug, serde::Serialize)]
pub struct ComponentScore {
    pub score: i32,
    pub weight: f64,
    pub weighted_contribution: f64,
    pub details: serde_json::Value,
}

#[derive(Debug, serde::Serialize)]
pub struct DocumentScore {
    pub score: i32,
    pub verified_documents: i32,
    pub latest_verification: Option<chrono::DateTime<Utc>>,
}

/// Score weights configuration
#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub on_chain_activity: f64,
    pub repayment_history: f64,
    pub collateral_quality: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            on_chain_activity: 0.40,
            repayment_history: 0.40,
            collateral_quality: 0.20,
        }
    }
}

/// Enhanced Risk Engine with DeFi and document verification
pub struct RiskEngineV2 {
    db_pool: PgPool,
    defi_provider: Option<Arc<DeFiHistoryProvider>>,
    doc_verification: Option<Arc<DocumentVerificationService>>,
    weights: ScoreWeights,
}

impl RiskEngineV2 {
    pub fn new(
        db_pool: PgPool,
        defi_provider: Option<Arc<DeFiHistoryProvider>>,
        doc_verification: Option<Arc<DocumentVerificationService>>,
        weights: Option<ScoreWeights>,
    ) -> Self {
        Self {
            db_pool,
            defi_provider,
            doc_verification,
            weights: weights.unwrap_or_default(),
        }
    }

    /// Get reference to database pool
    pub fn db_pool(&self) -> &PgPool {
        &self.db_pool
    }

    /// Calculate enhanced risk score with all components
    pub async fn calculate_risk_score_v2(
        &self,
        user_id: Uuid,
        wallet_address: &str,
    ) -> Result<RiskScoreResponseV2, ApiError> {
        tracing::info!(
            user_id = %user_id,
            wallet = %wallet_address,
            "Calculating enhanced risk score v2"
        );

        // Calculate all components concurrently
        let (on_chain_score, repayment_score, collateral_score, doc_score) = tokio::join!(
            self.calculate_on_chain_activity_score(wallet_address),
            self.calculate_repayment_history_score(user_id),
            self.calculate_collateral_quality_score(user_id),
            self.calculate_document_verification_score(user_id)
        );

        let on_chain_score = on_chain_score?;
        let repayment_score = repayment_score?;
        let collateral_score = collateral_score?;
        let doc_score = doc_score.ok();

        // Apply weighted algorithm
        let overall_score = self.calculate_weighted_score(
            on_chain_score.score,
            repayment_score.score,
            collateral_score.score,
        );

        // Calculate confidence based on data availability
        let confidence = self.calculate_confidence_v2(
            &on_chain_score,
            &repayment_score,
            &collateral_score,
            &doc_score,
        );

        // Determine risk tier
        let risk_tier = self.determine_risk_tier(overall_score);

        // Store in database
        self.store_risk_score(
            user_id,
            overall_score,
            &risk_tier,
            confidence,
            &on_chain_score,
            &repayment_score,
            &collateral_score,
            &doc_score,
        )
        .await?;

        // Build response
        let components = ScoreComponents {
            on_chain_activity: on_chain_score,
            repayment_history: repayment_score,
            collateral_quality: collateral_score,
            document_verification: doc_score,
        };

        Ok(RiskScoreResponseV2 {
            user_id,
            wallet_address: wallet_address.to_string(),
            overall_score,
            risk_tier,
            confidence,
            components,
            calculated_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
        })
    }

    /// Calculate on-chain activity score from DeFi protocols
    async fn calculate_on_chain_activity_score(
        &self,
        wallet_address: &str,
    ) -> Result<ComponentScore, ApiError> {
        let score = if let Some(ref provider) = self.defi_provider {
            let query = DeFiHistoryQuery {
                wallet_address: wallet_address.to_string(),
                days_back: 90,
                include_soroswap: true,
                include_aquarius: true,
            };

            match provider.fetch_activity_metrics(query).await {
                Ok(metrics) => {
                    // Score based on transaction count and volume
                    let tx_score = (metrics.aggregated.total_tx_count.min(100) * 5) as i32;
                    let volume_score = ((metrics.aggregated.total_volume_usd / 1000).min(500) as i32).min(500);
                    let diversity_score = metrics.aggregated.protocol_diversity_score;
                    let consistency_score = metrics.aggregated.activity_consistency_score;

                    let base_score = (tx_score + volume_score + diversity_score + consistency_score) / 4;

                    ComponentScore {
                        score: base_score.min(1000),
                        weight: self.weights.on_chain_activity,
                        weighted_contribution: base_score as f64 * self.weights.on_chain_activity,
                        details: serde_json::json!({
                            "total_transactions": metrics.aggregated.total_tx_count,
                            "total_volume_usd": metrics.aggregated.total_volume_usd,
                            "protocol_diversity": metrics.aggregated.protocol_diversity_score,
                            "activity_consistency": metrics.aggregated.activity_consistency_score,
                            "soroswap_tx": metrics.soroswap_metrics.tx_count,
                            "aquarius_tx": metrics.aquarius_metrics.tx_count,
                        }),
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch DeFi metrics: {}", e);
                    self.default_component_score(self.weights.on_chain_activity)
                }
            }
        } else {
            self.default_component_score(self.weights.on_chain_activity)
        };

        Ok(score)
    }

    /// Calculate repayment history score from loan data
    async fn calculate_repayment_history_score(
        &self,
        user_id: Uuid,
    ) -> Result<ComponentScore, ApiError> {
        let loan_stats: (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) as total_loans,
                COUNT(*) FILTER (WHERE status = 'repaid') as repaid_loans,
                COUNT(*) FILTER (WHERE status = 'defaulted') as defaulted_loans
            FROM loans
            WHERE borrower_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let (total_loans, repaid_loans, defaulted_loans) = loan_stats;

        let score = if total_loans > 0 {
            let repayment_rate = repaid_loans as f64 / total_loans as f64;
            let default_penalty = (defaulted_loans as f64 / total_loans as f64) * 500.0;
            
            let base_score = (repayment_rate * 1000.0 - default_penalty) as i32;
            base_score.max(0).min(1000)
        } else {
            500 // Neutral score for no history
        };

        Ok(ComponentScore {
            score,
            weight: self.weights.repayment_history,
            weighted_contribution: score as f64 * self.weights.repayment_history,
            details: serde_json::json!({
                "total_loans": total_loans,
                "repaid_loans": repaid_loans,
                "defaulted_loans": defaulted_loans,
                "repayment_rate": if total_loans > 0 { repaid_loans as f64 / total_loans as f64 } else { 0.0 },
            }),
        })
    }

    /// Calculate collateral quality score
    async fn calculate_collateral_quality_score(
        &self,
        user_id: Uuid,
    ) -> Result<ComponentScore, ApiError> {
        let collateral_stats: (i64, Option<i64>) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) as total_collateral,
                SUM(face_value) as total_value
            FROM collateral
            WHERE owner_id = $1 AND status = 'active'
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let (total_collateral, total_value) = collateral_stats;
        let total_value = total_value.unwrap_or(0);

        let score = if total_collateral > 0 {
            let count_score = (total_collateral.min(10) * 50) as i32;
            let value_score = ((total_value / 1000).min(500) as i32).min(500);
            (count_score + value_score).min(1000)
        } else {
            300 // Low score for no collateral
        };

        Ok(ComponentScore {
            score,
            weight: self.weights.collateral_quality,
            weighted_contribution: score as f64 * self.weights.collateral_quality,
            details: serde_json::json!({
                "total_collateral": total_collateral,
                "total_value": total_value,
                "average_value": if total_collateral > 0 { total_value / total_collateral } else { 0 },
            }),
        })
    }

    /// Calculate document verification score
    async fn calculate_document_verification_score(
        &self,
        user_id: Uuid,
    ) -> Result<DocumentScore, ApiError> {
        if let Some(ref doc_service) = self.doc_verification {
            let avg_score = doc_service
                .get_user_average_score(&user_id)
                .await
                .map_err(|e| ApiError::InternalError(e.to_string()))?;

            let verifications = doc_service
                .get_user_verifications(&user_id)
                .await
                .map_err(|e| ApiError::InternalError(e.to_string()))?;

            let latest_verification = verifications
                .first()
                .and_then(|v| v.verified_at);

            Ok(DocumentScore {
                score: avg_score.unwrap_or(0),
                verified_documents: verifications.len() as i32,
                latest_verification,
            })
        } else {
            Ok(DocumentScore {
                score: 0,
                verified_documents: 0,
                latest_verification: None,
            })
        }
    }

    /// Apply weighted algorithm: 40% onChain, 40% repayment, 20% collateral
    fn calculate_weighted_score(
        &self,
        on_chain: i32,
        repayment: i32,
        collateral: i32,
    ) -> i32 {
        let weighted = (on_chain as f64 * self.weights.on_chain_activity)
            + (repayment as f64 * self.weights.repayment_history)
            + (collateral as f64 * self.weights.collateral_quality);

        weighted.round() as i32
    }

    /// Calculate confidence based on data availability
    fn calculate_confidence_v2(
        &self,
        on_chain: &ComponentScore,
        repayment: &ComponentScore,
        collateral: &ComponentScore,
        doc_score: &Option<DocumentScore>,
    ) -> f64 {
        let mut confidence = 0.0;
        let mut factors = 0;

        // On-chain activity confidence
        if on_chain.score > 0 {
            confidence += 0.25;
            factors += 1;
        }

        // Repayment history confidence
        if repayment.score > 0 {
            confidence += 0.35;
            factors += 1;
        }

        // Collateral confidence
        if collateral.score > 0 {
            confidence += 0.25;
            factors += 1;
        }

        // Document verification bonus
        if let Some(doc) = doc_score {
            if doc.verified_documents > 0 {
                confidence += 0.15;
                factors += 1;
            }
        }

        if factors > 0 {
            confidence
        } else {
            0.1 // Minimum confidence
        }
    }

    /// Determine risk tier from score
    fn determine_risk_tier(&self, score: i32) -> String {
        match score {
            850..=1000 => "excellent".to_string(),
            700..=849 => "good".to_string(),
            550..=699 => "fair".to_string(),
            400..=549 => "poor".to_string(),
            _ => "very_poor".to_string(),
        }
    }

    /// Store risk score in database
    async fn store_risk_score(
        &self,
        user_id: Uuid,
        overall_score: i32,
        risk_tier: &str,
        confidence: f64,
        on_chain: &ComponentScore,
        repayment: &ComponentScore,
        collateral: &ComponentScore,
        doc_score: &Option<DocumentScore>,
    ) -> Result<Uuid, ApiError> {
        let risk_score_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO risk_scores (
                id, user_id, overall_score, risk_tier, confidence,
                on_chain_activity_score, repayment_history_score, collateral_quality_score,
                document_verification_score,
                on_chain_activity_weight, repayment_history_weight, collateral_quality_weight,
                calculation_version, calculated_at, expires_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4::risk_tier, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW() + INTERVAL '24 hours', NOW(), NOW())
            "#
        )
        .bind(risk_score_id)
        .bind(user_id)
        .bind(overall_score)
        .bind(risk_tier)
        .bind(confidence)
        .bind(on_chain.score)
        .bind(repayment.score)
        .bind(collateral.score)
        .bind(doc_score.as_ref().map(|d| d.score))
        .bind(self.weights.on_chain_activity)
        .bind(self.weights.repayment_history)
        .bind(self.weights.collateral_quality)
        .bind("v2.0")
        .execute(&self.db_pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(risk_score_id)
    }

    /// Default component score when data is unavailable
    fn default_component_score(&self, weight: f64) -> ComponentScore {
        ComponentScore {
            score: 0,
            weight,
            weighted_contribution: 0.0,
            details: serde_json::json!({"status": "no_data"}),
        }
    }
}
