use std::env;

#[derive(Clone, Debug)]
pub struct ContractsConfig {
    pub collateral_contract_id: String,
    pub escrow_contract_id: String,
    pub loan_contract_id: String,
    pub governance_contract_id: Option<String>,
}

impl ContractsConfig {
    pub fn from_env() -> Self {
        Self {
            collateral_contract_id: env::var("COLLATERAL_CONTRACT_ID").unwrap_or_default(),
            escrow_contract_id: env::var("ESCROW_CONTRACT_ID").unwrap_or_default(),
            loan_contract_id: env::var("LOAN_CONTRACT_ID").unwrap_or_default(),
            governance_contract_id: env::var("GOVERNANCE_CONTRACT_ID").ok(),
        }
    }

    pub fn monitored_contract_ids(&self) -> Vec<String> {
        let mut ids = vec![
            self.collateral_contract_id.clone(),
            self.escrow_contract_id.clone(),
            self.loan_contract_id.clone(),
        ];

        if let Some(governance_id) = &self.governance_contract_id {
            ids.push(governance_id.clone());
        }

        ids.into_iter().filter(|id| !id.trim().is_empty()).collect()
    }
}
