use cosmwasm_std::{QuerierWrapper, Addr, Uint128};

pub fn get_balance(querier: &QuerierWrapper, address: &Addr, denom: &str) -> Result<Uint128, cosmwasm_std::StdError> {
    Ok(querier.query_balance(address, denom)?.amount)
}
