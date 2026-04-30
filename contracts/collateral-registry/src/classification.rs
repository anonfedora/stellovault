//! Asset classification and categorization

use soroban_sdk::{contracttype, Address, String};

/// Asset classification
#[contracttype]
#[derive(Clone)]
pub struct AssetClassification {
    pub collateral_id: u64,
    pub primary_class: AssetClass,
    // `Unspecified` preserves "no secondary class" semantics without changing
    // the stored shape of this struct.
    pub secondary_class: AssetClass,
    pub risk_rating: RiskRating,
    pub liquidity_score: u32,
    pub classified_by: Address,
    pub classified_at: u64,
}

/// Asset class enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetClass {
    RealEstate = 0,
    Equipment = 1,
    Inventory = 2,
    Receivables = 3,
    Securities = 4,
    Commodities = 5,
    Vehicles = 6,
    Intellectual = 7,
    Other = 8,
    Unspecified = 255,
}

/// Risk rating enumeration
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum RiskRating {
    AAA = 0,
    AA = 1,
    A = 2,
    BBB = 3,
    BB = 4,
    B = 5,
    CCC = 6,
    CC = 7,
    C = 8,
    D = 9,
}

/// Real estate classification
#[contracttype]
#[derive(Clone)]
pub struct RealEstateClassification {
    pub collateral_id: u64,
    pub property_type: PropertyType,
    pub location: String,
    pub square_footage: u64,
    pub year_built: u32,
    pub zoning: String,
}

/// Property type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyType {
    Residential = 0,
    Commercial = 1,
    Industrial = 2,
    Agricultural = 3,
    Mixed = 4,
}

/// Equipment classification
#[contracttype]
#[derive(Clone)]
pub struct EquipmentClassification {
    pub collateral_id: u64,
    pub equipment_type: EquipmentType,
    pub manufacturer: String,
    pub model: String,
    pub year_manufactured: u32,
    pub condition: EquipmentCondition,
}

/// Equipment type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquipmentType {
    Machinery = 0,
    Vehicles = 1,
    Electronics = 2,
    Tools = 3,
    Other = 4,
}

/// Equipment condition
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EquipmentCondition {
    New = 0,
    Excellent = 1,
    Good = 2,
    Fair = 3,
    Poor = 4,
}

/// Inventory classification
#[contracttype]
#[derive(Clone)]
pub struct InventoryClassification {
    pub collateral_id: u64,
    pub inventory_type: InventoryType,
    pub quantity: u64,
    pub unit_value: i128,
    pub storage_location: String,
}

/// Inventory type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InventoryType {
    RawMaterials = 0,
    WorkInProgress = 1,
    FinishedGoods = 2,
    Supplies = 3,
    Other = 4,
}

/// Securities classification
#[contracttype]
#[derive(Clone)]
pub struct SecuritiesClassification {
    pub collateral_id: u64,
    pub security_type: SecurityType,
    pub issuer: String,
    pub quantity: u64,
    pub market_price: i128,
}

/// Security type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum SecurityType {
    Stock = 0,
    Bond = 1,
    MutualFund = 2,
    ETF = 3,
    Other = 4,
}

/// Commodities classification
#[contracttype]
#[derive(Clone)]
pub struct CommoditiesClassification {
    pub collateral_id: u64,
    pub commodity_type: CommodityType,
    pub quantity: u64,
    pub unit: String,
    pub market_price: i128,
}

/// Commodity type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommodityType {
    Metals = 0,
    Energy = 1,
    Agriculture = 2,
    Crypto = 3,
    Other = 4,
}

/// Intellectual property classification
#[contracttype]
#[derive(Clone)]
pub struct IntellectualPropertyClassification {
    pub collateral_id: u64,
    pub ip_type: IPType,
    pub registration_number: String,
    pub jurisdiction: String,
    pub expiry_date: u64,
}

/// IP type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IPType {
    Patent = 0,
    Trademark = 1,
    Copyright = 2,
    TradeSecret = 3,
    Other = 4,
}

/// Classification history
#[contracttype]
#[derive(Clone)]
pub struct ClassificationHistory {
    pub collateral_id: u64,
    pub classifications: soroban_sdk::Vec<ClassificationRecord>,
}

/// Classification record
#[contracttype]
#[derive(Clone)]
pub struct ClassificationRecord {
    pub primary_class: AssetClass,
    pub risk_rating: RiskRating,
    pub classified_by: Address,
    pub classified_at: u64,
}

/// Classification update
#[contracttype]
#[derive(Clone)]
pub struct ClassificationUpdate {
    pub id: u64,
    pub collateral_id: u64,
    pub old_classification: AssetClass,
    pub new_classification: AssetClass,
    pub reason: String,
    pub updated_by: Address,
    pub updated_at: u64,
}
