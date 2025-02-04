use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub swap_pool_address: String,
    pub tax_rate: u64, // Percentage in hundredths (25% = 2500)
}

pub const CONFIG: Item<Config> = Item::new("config");