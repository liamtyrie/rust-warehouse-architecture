use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvRow {
    pub shipment_id: u64,
    pub product_id: u64,
    pub quantity: u32,
}

#[derive(Serialize, Debug)]
pub struct RowWithUser {
    pub user_id: u64,
    pub shipment_id: u64,
    pub product_id: u64,
    pub quantity: u32,
}
