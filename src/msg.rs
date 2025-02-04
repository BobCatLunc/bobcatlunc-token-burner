use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{from_json, Binary};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub swap_pool_address: String,
    pub tax_rate: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive {},
    UpdateSwapPoolAddress { address: String },
	UpdateTaxRate { tax_rate: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
	// Implement if needed
}
