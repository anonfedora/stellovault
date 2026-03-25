//! Document validation and fraud detection logic

use super::model::*;

/// Validator for document verification
pub struct DocumentValidator {
    min_confidence: f64,
    #[allow(dead_code)]
    fraud_detection_level: FraudDetectionLevel,
}

#[derive(Debug, Clone)]
pub enum FraudDetectionLevel {
    Low,
    Medium,
    High,
}

impl DocumentValidator {
    pub fn new(min_confidence: f64, fraud_detection_level: FraudDetectionLevel) -> Self {
        Self {
            min_confidence,
            fraud_detection_level,
        }
    }

    /// Validate extracted fields and detect fraud
    pub fn validate_and_score(
        &self,
        extracted_fields: &ExtractedFields,
        confidence: f64,
    ) -> (i32, Vec<FraudIndicator>, bool) {
        let mut fraud_indicators = Vec::new();
        let mut base_score = 100;

        // Check confidence level
        if confidence < self.min_confidence {
            fraud_indicators.push(FraudIndicator {
                indicator_type: "low_confidence".to_string(),
                severity: FraudSeverity::Medium,
                description: format!("OCR confidence {} is below threshold {}", confidence, self.min_confidence),
                confidence: 1.0 - confidence,
            });
            base_score -= 20;
        }

        // Validate required fields
        if extracted_fields.invoice_number.is_none() {
            fraud_indicators.push(FraudIndicator {
                indicator_type: "missing_invoice_number".to_string(),
                severity: FraudSeverity::High,
                description: "Invoice number is missing".to_string(),
                confidence: 0.9,
            });
            base_score -= 30;
        }

        if extracted_fields.date.is_none() {
            fraud_indicators.push(FraudIndicator {
                indicator_type: "missing_date".to_string(),
                severity: FraudSeverity::Medium,
                description: "Invoice date is missing".to_string(),
                confidence: 0.8,
            });
            base_score -= 15;
        }

        if extracted_fields.amount.is_none() {
            fraud_indicators.push(FraudIndicator {
                indicator_type: "missing_amount".to_string(),
                severity: FraudSeverity::High,
                description: "Invoice amount is missing".to_string(),
                confidence: 0.9,
            });
            base_score -= 30;
        }

        if extracted_fields.vendor_name.is_none() {
            fraud_indicators.push(FraudIndicator {
                indicator_type: "missing_vendor".to_string(),
                severity: FraudSeverity::Medium,
                description: "Vendor name is missing".to_string(),
                confidence: 0.8,
            });
            base_score -= 15;
        }

        // Check for suspicious amounts
        if let Some(amount) = extracted_fields.amount {
            if amount <= 0.0 {
                fraud_indicators.push(FraudIndicator {
                    indicator_type: "invalid_amount".to_string(),
                    severity: FraudSeverity::Critical,
                    description: "Invoice amount is zero or negative".to_string(),
                    confidence: 1.0,
                });
                base_score -= 50;
            } else if amount > 1_000_000.0 {
                fraud_indicators.push(FraudIndicator {
                    indicator_type: "unusually_high_amount".to_string(),
                    severity: FraudSeverity::Medium,
                    description: format!("Invoice amount ${} is unusually high", amount),
                    confidence: 0.6,
                });
                base_score -= 10;
            }
        }

        // Check for duplicate invoice numbers (would need database check in production)
        // This is a placeholder for the logic
        
        // Determine if manual review is required
        let requires_manual_review = base_score < 70 || 
            fraud_indicators.iter().any(|fi| matches!(fi.severity, FraudSeverity::Critical));

        let final_score = base_score.max(0).min(100);

        (final_score, fraud_indicators, requires_manual_review)
    }

    /// Check if document should be auto-approved
    pub fn should_auto_approve(&self, score: i32, fraud_indicators: &[FraudIndicator]) -> bool {
        score >= 75 && !fraud_indicators.iter().any(|fi| {
            matches!(fi.severity, FraudSeverity::High | FraudSeverity::Critical)
        })
    }

    /// Determine verification status based on score and fraud indicators
    pub fn determine_status(
        &self,
        score: i32,
        requires_manual_review: bool,
    ) -> VerificationStatus {
        if requires_manual_review {
            VerificationStatus::ManualReview
        } else if score >= 75 {
            VerificationStatus::Verified
        } else if score >= 50 {
            VerificationStatus::ManualReview
        } else {
            VerificationStatus::Rejected
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_complete_document() {
        let validator = DocumentValidator::new(0.75, FraudDetectionLevel::Medium);
        
        let fields = ExtractedFields {
            invoice_number: Some("INV-001".to_string()),
            date: Some("2024-03-25".to_string()),
            amount: Some(1500.0),
            currency: Some("USD".to_string()),
            vendor_name: Some("Acme Corp".to_string()),
            vendor_address: None,
            line_items: vec![],
        };

        let (score, fraud_indicators, requires_review) = validator.validate_and_score(&fields, 0.95);
        
        assert_eq!(score, 100);
        assert_eq!(fraud_indicators.len(), 0);
        assert!(!requires_review);
    }

    #[test]
    fn test_validate_missing_fields() {
        let validator = DocumentValidator::new(0.75, FraudDetectionLevel::Medium);
        
        let fields = ExtractedFields {
            invoice_number: None,
            date: None,
            amount: None,
            currency: None,
            vendor_name: None,
            vendor_address: None,
            line_items: vec![],
        };

        let (score, fraud_indicators, requires_review) = validator.validate_and_score(&fields, 0.95);
        
        assert!(score < 50);
        assert!(fraud_indicators.len() >= 4);
        assert!(requires_review);
    }

    #[test]
    fn test_validate_invalid_amount() {
        let validator = DocumentValidator::new(0.75, FraudDetectionLevel::Medium);
        
        let fields = ExtractedFields {
            invoice_number: Some("INV-001".to_string()),
            date: Some("2024-03-25".to_string()),
            amount: Some(-100.0),
            currency: Some("USD".to_string()),
            vendor_name: Some("Acme Corp".to_string()),
            vendor_address: None,
            line_items: vec![],
        };

        let (score, fraud_indicators, requires_review) = validator.validate_and_score(&fields, 0.95);
        
        assert!(score < 70);
        assert!(fraud_indicators.iter().any(|fi| fi.indicator_type == "invalid_amount"));
        assert!(requires_review);
    }
}
