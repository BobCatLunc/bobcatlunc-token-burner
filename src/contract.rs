use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_json_binary, BankMsg, Coin, SubMsg, WasmMsg, entry_point, Uint128, Binary, Reply, from_json};
use crate::state::{Config, CONFIG};
use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
use crate::error::ContractError;
use crate::utils::{get_balance};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admin: info.sender,
        swap_pool_address: msg.swap_pool_address,
        tax_rate: msg.tax_rate,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
	
	// Check if funds were sent with the message and if they include uluna
	if !info.funds.is_empty() {
		if let Some(_) = info.funds.iter().find(|c| c.denom == "uluna") {
			return handle_receive(deps, env, info);
		}else if let Some(_) = info.funds.iter().find(|c| c.denom == "uusd") {
			return handle_receive_uusd(deps, env, info);
		}
	}
	
    match msg {
        ExecuteMsg::Receive {} => handle_receive(deps, env, info),
        ExecuteMsg::UpdateSwapPoolAddress { address } => handle_update_swap_pool_address(deps, info, address),
		ExecuteMsg::UpdateTaxRate { tax_rate } => handle_update_tax_rate(deps, info, tax_rate),
    }
}

// New function to handle deserialization errors
pub fn try_execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Binary,
) -> Result<Response, ContractError> {
    // Attempt to deserialize the message
    match from_json(msg) {
        Ok(execute_msg) => execute(deps, env, info, execute_msg),
        Err(_) => {
            if !info.funds.is_empty() {
                // If deserialization fails but funds are sent, treat as receive
                handle_receive(deps, env, info)
            } else {
                // If no funds and deserialization fails, return an error
                Err(ContractError::InvalidMessage {})
            }
        }
    }
}

fn handle_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let uluna_amount = info.funds.into_iter().find(|c| c.denom == "uluna")
        .ok_or(ContractError::NoLunaReceived {})?
        .amount;
    
    let amount_to_swap = uluna_amount * Uint128::from(config.tax_rate as u64) / Uint128::from(10000u64);
    
    let swap_msg = WasmMsg::Execute {
        contract_addr: config.swap_pool_address.to_string(),
        msg: to_json_binary(&serde_json::json!({ "swap": { "offer_asset": { "info": { "native_token": { "denom": "uluna" } }, "amount": amount_to_swap.to_string() }, "to": env.contract.address.to_string() } }))?,
        funds: vec![Coin{ denom: "uluna".to_string(), amount: amount_to_swap }],
    };
    
    let burn_address = "terra1sk06e3dyexuq4shw77y3dsv480xv42mq73anxu".to_string();

    // Use reply mechanism for both success and failure cases
    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(swap_msg, 1)) // ID 1 for successful swap
        .add_submessage(SubMsg::reply_on_error(
            BankMsg::Send {
                to_address: burn_address.clone(),
                amount: vec![Coin{ denom: "uluna".to_string(), amount: uluna_amount }],
            },
            3 // ID 3 for swap failure
        ))
        .add_attribute("action", "initiate_swap")
        .add_attribute("memo", "LUNC_BURN_WITH_USTC_BUYBACK_N_BURN"))
}

fn handle_receive_uusd(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let uusd_amount = info.funds.into_iter().find(|c| c.denom == "uusd")
        .ok_or(ContractError::NoUstcReceived {})?
        .amount;
    
    let tax_rate = if config.tax_rate > 10000 { 10000 } else { config.tax_rate };
    let amount_to_swap = uusd_amount * Uint128::from( (10000 - tax_rate) as u64) / Uint128::from(10000u64);
    
    let swap_msg = WasmMsg::Execute {
        contract_addr: config.swap_pool_address.to_string(),
        msg: to_json_binary(&serde_json::json!({ "swap": { "offer_asset": { "info": { "native_token": { "denom": "uusd" } }, "amount": amount_to_swap.to_string() }, "to": env.contract.address.to_string() } }))?,
        funds: vec![Coin{ denom: "uusd".to_string(), amount: amount_to_swap }],
    };
    
    let burn_address = "terra1sk06e3dyexuq4shw77y3dsv480xv42mq73anxu".to_string();

    // Use reply mechanism for both success and failure cases
    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(swap_msg, 2)) // ID 2 for successful USTC swap
        .add_submessage(SubMsg::reply_on_error(
            BankMsg::Send {
                to_address: burn_address.clone(),
                amount: vec![Coin{ denom: "uusd".to_string(), amount: uusd_amount }],
            },
            4 // ID 4 for USTC swap failure
        ))
        .add_attribute("action", "initiate_swap")
        .add_attribute("memo", "LUNC_BURN_WITH_USTC_BUYBACK_N_BURN"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 | 2 => {
            // Determine token based on reply ID
            let token_denom = if msg.id == 1 { "uluna" } else { "uusd" };
            handle_swap_reply(deps, env, token_denom)
        },
        3 => {
            // Swap for uluna failed, forward all uluna (This could be handled by a general error handler if needed)
            Ok(Response::new()
                .add_attribute("action", "swap_failed_burn")
                .add_attribute("memo", "LUNC_BURN"))
        },
        4 => {
            // Swap for uusd failed, forward all uusd (This could be handled by a general error handler if needed)
            Ok(Response::new()
                .add_attribute("action", "swap_failed_burn")
                .add_attribute("memo", "USTC_BURN"))
        },
        _ => Err(ContractError::UnknownReplyId { id: msg.id }),
    }
}

fn handle_swap_reply(
    deps: DepsMut,
    env: Env,
    token_denom: &str,
) -> Result<Response, ContractError> {
    let burn_address = "terra1sk06e3dyexuq4shw77y3dsv480xv42mq73anxu".to_string();
    let balance = get_balance(&deps.querier, &env.contract.address, token_denom)?;
    let other_balance = get_balance(&deps.querier, &env.contract.address, if token_denom == "uluna" { "uusd" } else { "uluna" })?;

    let burn_msg = BankMsg::Send {
        to_address: burn_address,
        amount: vec![
            Coin{ denom: token_denom.to_string(), amount: balance },
            Coin{ denom: if token_denom == "uluna" { "uusd".to_string() } else { "uluna".to_string() }, amount: other_balance }
        ],
    };

    Ok(Response::new()
        .add_message(burn_msg)
        .add_attribute("action", "burn_after_swap")
        .add_attribute("memo", "LUNC_BURN_WITH_USTC_BUYBACK_N_BURN"))
}

fn handle_update_swap_pool_address(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    config.swap_pool_address = address;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "update_swap_pool_address"))
}

fn handle_update_tax_rate(
    deps: DepsMut,
    info: MessageInfo,
    tax_rate: u64,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Ensure tax_rate is within reasonable bounds (e.g., 0 to 10000 for 0% to 100%)
    if tax_rate > 10000 {
        return Err(ContractError::InvalidTaxRate {});
    }

    config.tax_rate = tax_rate;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_tax_rate")
        .add_attribute("new_tax_rate", tax_rate.to_string()))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        // Add other query messages here
    }.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr, Response, BankMsg, Coin, WasmMsg, SubMsg};

    #[test]
    fn test_execute_with_uluna_directly() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("sender", &coins(1000000, "uluna")); // 1 LUNC

        let msg = InstantiateMsg {
            swap_pool_address: "some_swap_pool_address".to_string(),
            tax_rate: 2500, // 25%
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Here, we simulate direct receipt of funds without an explicit ExecuteMsg
        let res = execute(deps.as_mut(), env.clone(), info.clone(), ExecuteMsg::Receive {}).unwrap();

        // Check if handle_receive was executed by examining the response:
        // - Check for the swap message
        assert_eq!(res.messages.len(), 2); // Expecting one for swap, one for potential burn after swap
        match &res.messages[0] {
            SubMsg { msg, id, gas_limit, reply_on } => {
                match msg {
                    cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, funds }) => {
                        assert_eq!(contract_addr, "some_swap_pool_address");
                        assert_eq!(funds.len(), 1);
                        assert_eq!(funds[0].denom, "uluna");
                        assert_eq!(funds[0].amount, Uint128::from(250000u128)); // 25% of 1,000,000
                    },
                    _ => panic!("Unexpected message type"),
                }
            }
        }

        // Check if the attribute was added to indicate handle_receive was executed (adjust based on your implementation)
        assert!(res.attributes.iter().any(|a| a.key == "action" && a.value == "initiate_swap"));
    }

    #[test]
    fn test_execute_with_uusd_directly() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("sender", &coins(1000000, "uusd")); // 1 USTC

        let msg = InstantiateMsg {
            swap_pool_address: "some_swap_pool_address".to_string(),
            tax_rate: 5000, // 50%, meaning 50% to swap for USTC
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Simulate direct receipt of funds without an explicit ExecuteMsg
        let res = execute(deps.as_mut(), env.clone(), info.clone(), ExecuteMsg::Receive {}).unwrap();

        // Check if handle_receive_uusd was executed by examining the response:
        assert_eq!(res.messages.len(), 2); // Expecting one for swap, one for potential burn after swap
        match &res.messages[0] {
            SubMsg { msg, id, gas_limit, reply_on } => {
                match msg {
                    cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, funds }) => {
                        assert_eq!(contract_addr, "some_swap_pool_address");
                        assert_eq!(funds.len(), 1);
                        assert_eq!(funds[0].denom, "uusd");
                        assert_eq!(funds[0].amount, Uint128::from(500000u128)); // 50% of 1,000,000
                    },
                    _ => panic!("Unexpected message type"),
                }
            }
        }

        // Check if the attribute was added to indicate handle_receive_uusd was executed (adjust based on your implementation)
        assert!(res.attributes.iter().any(|a| a.key == "action" && a.value == "initiate_swap"));
    }

    // You might also want to add tests for:
    // - Sending both uluna and uusd at once, to see which function gets priority or how it's handled.
    // - Sending tokens other than uluna or uusd to ensure they're not processed.
    // - Sending no funds to ensure handle_receive isn't called unnecessarily.
}