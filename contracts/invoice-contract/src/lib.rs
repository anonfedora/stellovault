//! Invoice Tokenization Contract for StelloVault
//!
//! This contract enables tokenization of trade invoices with:
//! - Invoice verification and authentication
//! - Payment terms management
//! - Automated payment processing
//! - Invoice discounting and factoring
//! - Fraud detection
//! - Cross-border payment support
//! - Supply chain finance features

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, String, Symbol,
    Vec, Map,
};

mod invoice;
mod payment;
mod verification;
mod fraud_detection;
mod cross_border;

pub use invoice::*;
pub use payment::*;
pub use verification::*;
pub use fraud_detection::*;
pub use cross_border::*;

/// Contract errors
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    AlreadyInitialized = 2,
    InvalidInvoiceData = 3,
    InvoiceNotFound = 4,
    InvoiceNotVerified = 5,
    InvoiceExpired = 6,
    InvoiceAlreadyPaid = 7,
    InvalidPaymentAmount = 8,
    PaymentTermsNotMet = 9,
    FraudDetected = 10,
    InvalidVerificationData = 11,
    InsufficientFunds = 12,
    TransferFailed = 13,
    InvalidCurrency = 14,
    ExchangeRateError = 15,
    InvalidDiscount = 16,
    PaymentProcessingFailed = 17,
    VerificationExpired = 18,
    DuplicateInvoice = 19,
    InvalidOwnerTransfer = 20,
}

impl From<soroban_sdk::Error> for ContractError {
    fn from(_: soroban_sdk::Error) -> Self {
        ContractError::Unauthorized
    }
}

impl From<&ContractError> for soroban_sdk::Error {
    fn from(err: &ContractError) -> Self {
        soroban_sdk::Error::from_contract_error(*err as u32)
    }
}

/// Main invoice contract
#[contract]
pub struct InvoiceContract;

/// Contract implementation
#[contractimpl]
impl InvoiceContract {
    /// Initialize the contract with admin address
    ///
    /// # Arguments
    /// * `admin` - The admin address that can manage the contract
    /// * `treasury` - Treasury address for fee collection
    ///
    /// # Events
    /// Emits `ContractInitialized` event
    pub fn initialize(env: Env, admin: Address, treasury: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(ContractError::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &admin);
        env.storage()
            .instance()
            .set(&symbol_short!("treasury"), &treasury);
        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &1u64);

        // Initialize fraud detection system
        env.storage()
            .instance()
            .set(&symbol_short!("fraud_threshold"), &800u32); // 80% risk threshold

        env.events().publish(
            (symbol_short!("inv_init"),),
            (admin.clone(), treasury),
        );

        Ok(())
    }

    /// Tokenize a new invoice
    ///
    /// # Arguments
    /// * `issuer` - Address of the invoice issuer
    /// * `invoice_data` - Invoice data structure
    /// * `verification_hash` - SHA-256 hash of verification document
    ///
    /// # Returns
    /// The invoice ID
    ///
    /// # Events
    /// Emits `InvoiceTokenized` event
    pub fn tokenize_invoice(
        env: Env,
        issuer: Address,
        invoice_data: InvoiceData,
        verification_hash: BytesN<32>,
    ) -> Result<u64, ContractError> {
        issuer.require_auth();

        // Validate invoice data
        if invoice_data.amount <= 0 {
            return Err(ContractError::InvalidInvoiceData);
        }

        if invoice_data.due_date <= env.ledger().timestamp() {
            return Err(ContractError::InvalidInvoiceData);
        }

        // Check for duplicate invoices
        let invoice_key = format_invoice_key(&invoice_data.invoice_number);
        if env.storage().persistent().has(&invoice_key) {
            return Err(ContractError::DuplicateInvoice);
        }

        // Get next invoice ID
        let invoice_id: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("next_id"))
            .unwrap_or(1u64);

        // Create invoice record
        let invoice = Invoice {
            id: invoice_id,
            issuer: issuer.clone(),
            buyer: invoice_data.buyer.clone(),
            amount: invoice_data.amount,
            currency: invoice_data.currency.clone(),
            invoice_number: invoice_data.invoice_number.clone(),
            issue_date: env.ledger().timestamp(),
            due_date: invoice_data.due_date,
            description: invoice_data.description.clone(),
            verification_hash,
            verified: false,
            verification_timestamp: 0,
            paid_amount: 0,
            status: InvoiceStatus::Pending,
            owner: issuer.clone(),
            created_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
        };

        // Store invoice
        let storage_key = format_invoice_storage_key(invoice_id);
        env.storage().persistent().set(&storage_key, &invoice);

        // Store invoice number mapping
        env.storage()
            .persistent()
            .set(&invoice_key, &invoice_id);

        // Update next ID
        env.storage()
            .instance()
            .set(&symbol_short!("next_id"), &(invoice_id + 1));

        // Emit event
        env.events().publish(
            (symbol_short!("inv_tokenized"),),
            (invoice_id, issuer, invoice_data.amount),
        );

        Ok(invoice_id)
    }

    /// Verify invoice authenticity
    ///
    /// # Arguments
    /// * `invoice_id` - ID of the invoice to verify
    /// * `verification_data` - Verification data structure
    ///
    /// # Returns
    /// Verification result
    ///
    /// # Events
    /// Emits `InvoiceVerified` event
    pub fn verify_invoice(
        env: Env,
        invoice_id: u64,
        verification_data: VerificationData,
    ) -> Result<bool, ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        // Get invoice
        let storage_key = format_invoice_storage_key(invoice_id);
        let mut invoice: Invoice = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::InvoiceNotFound)?;

        // Verify document hash
        if invoice.verification_hash != verification_data.document_hash {
            return Err(ContractError::InvalidVerificationData);
        }

        // Check verification expiry
        if verification_data.verification_timestamp + 86400 * 30 < env.ledger().timestamp() {
            return Err(ContractError::VerificationExpired);
        }

        // Perform fraud detection
        let fraud_score = detect_fraud(&env, &invoice, &verification_data)?;
        let fraud_threshold: u32 = env
            .storage()
            .instance()
            .get(&symbol_short!("fraud_threshold"))
            .unwrap_or(800);

        if fraud_score > fraud_threshold {
            return Err(ContractError::FraudDetected);
        }

        // Update invoice status
        invoice.verified = true;
        invoice.verification_timestamp = env.ledger().timestamp();
        invoice.status = InvoiceStatus::Verified;
        invoice.updated_at = env.ledger().timestamp();

        // Store updated invoice
        env.storage().persistent().set(&storage_key, &invoice);

        // Emit event
        env.events().publish(
            (symbol_short!("inv_verified"),),
            (invoice_id, fraud_score),
        );

        Ok(true)
    }

    /// Set payment terms for an invoice
    ///
    /// # Arguments
    /// * `invoice_id` - ID of the invoice
    /// * `terms` - Payment terms structure
    ///
    /// # Events
    /// Emits `PaymentTermsSet` event
    pub fn set_payment_terms(
        env: Env,
        invoice_id: u64,
        terms: PaymentTerms,
    ) -> Result<(), ContractError> {
        // Get invoice
        let storage_key = format_invoice_storage_key(invoice_id);
        let mut invoice: Invoice = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::InvoiceNotFound)?;

        // Only owner can set payment terms
        let caller: Address = env.current_contract_address();
        if invoice.owner != caller && invoice.issuer != caller {
            return Err(ContractError::Unauthorized);
        }

        // Validate terms
        if terms.discount_rate > 10000 {
            // Max 100% discount
            return Err(ContractError::InvalidDiscount);
        }

        // Store payment terms
        let terms_key = format_payment_terms_key(invoice_id);
        env.storage().persistent().set(&terms_key, &terms);

        // Update invoice
        invoice.updated_at = env.ledger().timestamp();
        env.storage().persistent().set(&storage_key, &invoice);

        // Emit event
        env.events().publish(
            (symbol_short!("terms_set"),),
            (invoice_id, terms.discount_rate),
        );

        Ok(())
    }

    /// Process payment for an invoice
    ///
    /// # Arguments
    /// * `invoice_id` - ID of the invoice
    /// * `amount` - Payment amount
    /// * `payer` - Address of the payer
    ///
    /// # Events
    /// Emits `PaymentProcessed` event
    pub fn process_invoice_payment(
        env: Env,
        invoice_id: u64,
        amount: i128,
        payer: Address,
    ) -> Result<(), ContractError> {
        payer.require_auth();

        // Get invoice
        let storage_key = format_invoice_storage_key(invoice_id);
        let mut invoice: Invoice = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::InvoiceNotFound)?;

        // Verify invoice is verified
        if !invoice.verified {
            return Err(ContractError::InvoiceNotVerified);
        }

        // Check if already paid
        if invoice.status == InvoiceStatus::Paid {
            return Err(ContractError::InvoiceAlreadyPaid);
        }

        // Check if expired
        if invoice.due_date < env.ledger().timestamp() {
            return Err(ContractError::InvoiceExpired);
        }

        // Validate payment amount
        if amount <= 0 || amount > invoice.amount - invoice.paid_amount {
            return Err(ContractError::InvalidPaymentAmount);
        }

        // Get payment terms
        let terms_key = format_payment_terms_key(invoice_id);
        let terms: PaymentTerms = env
            .storage()
            .persistent()
            .get(&terms_key)
            .unwrap_or_else(|| PaymentTerms {
                discount_rate: 0,
                early_payment_discount: 0,
                late_payment_fee: 0,
                payment_deadline: invoice.due_date,
            });

        // Check payment terms
        if env.ledger().timestamp() > terms.payment_deadline {
            return Err(ContractError::PaymentTermsNotMet);
        }

        // Calculate effective payment amount with discounts
        let effective_amount = calculate_effective_payment(&env, amount, &terms, &invoice)?;

        // Update invoice
        invoice.paid_amount += effective_amount;
        invoice.updated_at = env.ledger().timestamp();

        if invoice.paid_amount >= invoice.amount {
            invoice.status = InvoiceStatus::Paid;
        } else {
            invoice.status = InvoiceStatus::PartiallyPaid;
        }

        // Store updated invoice
        env.storage().persistent().set(&storage_key, &invoice);

        // Record payment
        let payment_record = PaymentRecord {
            invoice_id,
            payer: payer.clone(),
            amount: effective_amount,
            timestamp: env.ledger().timestamp(),
            currency: invoice.currency.clone(),
        };

        let payment_key = format_payment_record_key(invoice_id, env.ledger().timestamp());
        env.storage().persistent().set(&payment_key, &payment_record);

        // Emit event
        env.events().publish(
            (symbol_short!("payment_processed"),),
            (invoice_id, effective_amount),
        );

        Ok(())
    }

    /// Calculate present value of invoice with discount
    ///
    /// # Arguments
    /// * `invoice_id` - ID of the invoice
    /// * `discount_rate` - Annual discount rate (basis points)
    ///
    /// # Returns
    /// Present value of the invoice
    pub fn calculate_invoice_value(
        env: Env,
        invoice_id: u64,
        discount_rate: u32,
    ) -> Result<i128, ContractError> {
        // Get invoice
        let storage_key = format_invoice_storage_key(invoice_id);
        let invoice: Invoice = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::InvoiceNotFound)?;

        // Calculate days to maturity
        let current_time = env.ledger().timestamp();
        let days_to_maturity = if invoice.due_date > current_time {
            (invoice.due_date - current_time) / 86400
        } else {
            0
        };

        // Calculate present value: PV = FV / (1 + r*t)
        let remaining_amount = invoice.amount - invoice.paid_amount;
        let discount_factor = 10000 + (discount_rate as u64 * days_to_maturity as u64 / 365);
        let present_value = (remaining_amount as u64 * 10000 / discount_factor) as i128;

        Ok(present_value)
    }

    /// Transfer invoice ownership
    ///
    /// # Arguments
    /// * `invoice_id` - ID of the invoice
    /// * `new_owner` - Address of the new owner
    ///
    /// # Events
    /// Emits `InvoiceTransferred` event
    pub fn transfer_invoice(
        env: Env,
        invoice_id: u64,
        new_owner: Address,
    ) -> Result<(), ContractError> {
        // Get invoice
        let storage_key = format_invoice_storage_key(invoice_id);
        let mut invoice: Invoice = env
            .storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::InvoiceNotFound)?;

        // Only current owner can transfer
        invoice.owner.require_auth();

        // Validate new owner
        if new_owner == invoice.owner {
            return Err(ContractError::InvalidOwnerTransfer);
        }

        // Update ownership
        let previous_owner = invoice.owner.clone();
        invoice.owner = new_owner.clone();
        invoice.updated_at = env.ledger().timestamp();

        // Store updated invoice
        env.storage().persistent().set(&storage_key, &invoice);

        // Emit event
        env.events().publish(
            (symbol_short!("inv_transferred"),),
            (invoice_id, previous_owner, new_owner),
        );

        Ok(())
    }

    /// Get invoice details
    ///
    /// # Arguments
    /// * `invoice_id` - ID of the invoice
    ///
    /// # Returns
    /// Invoice details
    pub fn get_invoice(env: Env, invoice_id: u64) -> Result<Invoice, ContractError> {
        let storage_key = format_invoice_storage_key(invoice_id);
        env.storage()
            .persistent()
            .get(&storage_key)
            .ok_or(ContractError::InvoiceNotFound)
    }

    /// Get payment terms for an invoice
    ///
    /// # Arguments
    /// * `invoice_id` - ID of the invoice
    ///
    /// # Returns
    /// Payment terms
    pub fn get_payment_terms(env: Env, invoice_id: u64) -> Result<PaymentTerms, ContractError> {
        let terms_key = format_payment_terms_key(invoice_id);
        env.storage()
            .persistent()
            .get(&terms_key)
            .ok_or(ContractError::InvalidInvoiceData)
    }

    /// Get invoice payment history
    ///
    /// # Arguments
    /// * `invoice_id` - ID of the invoice
    ///
    /// # Returns
    /// Vector of payment records
    pub fn get_payment_history(
        env: Env,
        invoice_id: u64,
    ) -> Result<Vec<PaymentRecord>, ContractError> {
        // Note: In production, this would iterate through payment records
        // For now, return empty vector as placeholder
        Ok(Vec::new(&env))
    }

    /// Update fraud detection threshold
    ///
    /// # Arguments
    /// * `new_threshold` - New fraud threshold (basis points)
    pub fn update_fraud_threshold(
        env: Env,
        new_threshold: u32,
    ) -> Result<(), ContractError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(ContractError::Unauthorized)?;

        admin.require_auth();

        if new_threshold > 10000 {
            return Err(ContractError::InvalidInvoiceData);
        }

        env.storage()
            .instance()
            .set(&symbol_short!("fraud_threshold"), &new_threshold);

        env.events()
            .publish((symbol_short!("fraud_threshold_updated"),), (new_threshold,));

        Ok(())
    }
}

// Helper functions

fn format_invoice_storage_key(invoice_id: u64) -> String {
    // In production, use proper key formatting
    String::from_slice(&Env::default(), &format!("invoice_{}", invoice_id))
}

fn format_invoice_key(invoice_number: &String) -> String {
    // In production, use proper key formatting
    String::from_slice(&Env::default(), &format!("inv_num_{}", invoice_number))
}

fn format_payment_terms_key(invoice_id: u64) -> String {
    String::from_slice(&Env::default(), &format!("terms_{}", invoice_id))
}

fn format_payment_record_key(invoice_id: u64, timestamp: u64) -> String {
    String::from_slice(
        &Env::default(),
        &format!("payment_{}_{}", invoice_id, timestamp),
    )
}

fn calculate_effective_payment(
    env: &Env,
    amount: i128,
    terms: &PaymentTerms,
    invoice: &Invoice,
) -> Result<i128, ContractError> {
    let mut effective_amount = amount;

    // Apply early payment discount
    if env.ledger().timestamp() < terms.payment_deadline {
        let days_early = (terms.payment_deadline - env.ledger().timestamp()) / 86400;
        if days_early > 0 {
            let discount = (amount as u64 * terms.early_payment_discount as u64 / 10000) as i128;
            effective_amount = amount - discount;
        }
    }

    Ok(effective_amount)
}
