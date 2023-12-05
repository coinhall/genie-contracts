use std::convert::TryInto;

use crate::crypto::is_valid_signature;
use crate::state::{Config, State, CONFIG, LIST_OF_IDS, STATE, USERS};
use cosmwasm_std::{
    attr, entry_point, from_json, to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty,
    Env, MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use genie::airdrop_nft::{
    ClaimNftPayload, ClaimResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse, Status,
    StatusResponse, UserInfoResponse,
};
use sha3::{Digest, Keccak256};
// use cw_ownable::{Action, Ownership, OwnershipError};

const CONTRACT_NAME: &str = "genie-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    if msg.from_timestamp <= env.block.time.seconds() {
        return Err(StdError::generic_err(
            "from_timestamp must be greater than current time",
        ));
    }
    if msg.to_timestamp <= msg.from_timestamp {
        return Err(StdError::generic_err(
            "to_timestamp must be greater than from_timestamp",
        ));
    }
    if msg.allocated_amounts.is_empty() {
        return Err(StdError::generic_err("allocated_amounts must not be empty"));
    }
    if msg.allocated_amounts.iter().any(|&x| x == Uint128::zero()) {
        return Err(StdError::generic_err(
            "allocated_amounts must not contain zero",
        ));
    }

    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        asset: msg.asset,
        from_timestamp: msg.from_timestamp,
        to_timestamp: msg.to_timestamp,
        allocated_amounts: msg.allocated_amounts.clone(),
        public_key: msg.public_key,
        mission_count: msg.allocated_amounts.len() as u64,
    };
    CONFIG.save(deps.storage, &config)?;
    let state = State {
        unclaimed_amounts: msg.allocated_amounts.clone(),
    };
    STATE.save(deps.storage, &state)?;

    // create a list of ids and store it in LIST_OF_IDS
    let end_id = msg.allocated_amounts.iter().sum::<Uint128>();

    for i in 0..end_id.u128() {
        let id = i.to_string();
        LIST_OF_IDS.save(deps.storage, id.to_string(), &Empty {})?;
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::Claim { payload } => handle_claim(deps, env, info, payload),
        ExecuteMsg::ReturnOwnership { recipient } => {
            handle_transfer_ownership(deps, env, info, recipient)
        }
        ExecuteMsg::IncreaseIncentives { topup_amounts } => {
            handle_increase_incentives(deps, env, info, topup_amounts)
        }
        ExecuteMsg::ReceiveOwnership {} => handle_receive_ownership(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::State {} => to_json_binary(&query_state(deps, env)?),
        QueryMsg::HasUserClaimed { address } => {
            to_json_binary(&query_has_user_claimed(deps, address)?)
        }
        QueryMsg::UserInfo { address } => to_json_binary(&query_user_info(deps, address)?),
        QueryMsg::Status {} => to_json_binary(&query_status(deps, &env)?),
    }
}

pub fn handle_receive_ownership(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, StdError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let messages = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.asset.contract_addr.to_string(),
        msg: to_json_binary(&cw_ownable::Action::AcceptOwnership {})?,
        funds: vec![],
    })];

    Result::Ok(
        Response::new()
            .add_attribute("action", "receive_ownership")
            .add_messages(messages),
    )
}

pub fn handle_transfer_ownership(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: Addr,
) -> Result<Response, StdError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let messages = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.asset.contract_addr.to_string(),
        msg: to_json_binary(&cw_ownable::Action::TransferOwnership {
            new_owner: recipient.to_string(),
            expiry: None,
        })?,
        funds: vec![],
    })];

    Result::Ok(
        Response::new()
            .add_attribute("action", "return_ownership")
            .add_messages(messages),
    )
}

pub fn handle_increase_incentives(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    topup_amounts: Option<Vec<Uint128>>,
) -> Result<Response, StdError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut state = STATE.load(deps.storage)?;
    let mut unclaimed_amounts = state.unclaimed_amounts;

    if let Some(topup_amounts) = topup_amounts {
        if topup_amounts.len() != unclaimed_amounts.len() {
            return Err(StdError::generic_err(
                "topup amount length does not match claimable amount length",
            ));
        }
        for (i, amount) in topup_amounts.iter().enumerate() {
            unclaimed_amounts[i] = unclaimed_amounts[i].checked_add(*amount)?;
        }
    }

    state.unclaimed_amounts = unclaimed_amounts;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("action", "increase_incentives"))
}

// Test using this for now, change it later.
pub fn generate_nft_ids_indexes(
    deps: DepsMut,
    msg: String,
    amount_to_generate: Uint128,
    max_index: Uint128,
) -> Result<Vec<String>, StdError> {
    // let msg_buf = msg.as_bytes();
    // let keccak_digest = Keccak256::digest(msg_buf);
    // let hash: &[u8] = keccak_digest.as_slice();
    // let max_index: u128 = max_index.into();

    // // hash modulo max_index to get 1 index
    // let mut index: u128 = u128::from_le_bytes([
    //     hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7], hash[8], hash[9],
    //     hash[10], hash[11], hash[12], hash[13], hash[14], hash[15],
    // ]) % max_index;

    // // range from index to index + amount_to_generate
    // let mut indexes: Vec<u128> = vec![];
    // for _ in 0..amount_to_generate.into() {
    //     indexes.push(index);
    //     index += 1;
    //     if index >= max_index {
    //         index = 0;
    //     }
    // }
    let amount: usize = amount_to_generate.u128().try_into().unwrap();

    LIST_OF_IDS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .take(amount)
        .map(|item| {
            let (key, _) = item?;
            Ok(key)
        })
        .collect::<StdResult<Vec<String>>>()
}

pub fn handle_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payload: Binary,
) -> Result<Response, StdError> {
    let ClaimNftPayload {
        claim_amounts,
        signature,
    } = from_json(&payload)?;
    if query_status(deps.as_ref(), &env)?.status != Status::Ongoing {
        return Err(StdError::generic_err("campaign is not ongoing"));
    }

    let recipient = &info.sender;
    let config = &CONFIG.load(deps.storage)?;
    let mut user_info = USERS.load(deps.storage, recipient).unwrap_or_default();
    let mut state = STATE.load(deps.storage)?;

    if claim_amounts.len() != state.unclaimed_amounts.len() {
        return Err(StdError::generic_err(
            "claim amount length does not match claimable amount length",
        ));
    }

    // Check if claim_amounts signature is valid
    let is_valid = is_valid_signature(
        &deps,
        recipient,
        &env.contract.address.to_string(),
        claim_amounts.clone(),
        &signature,
        &config.public_key,
    )?;
    if !is_valid {
        return Err(StdError::generic_err("signature verification failed"));
    }

    let mut claimable_amounts: Vec<Uint128> = vec![];
    // Iterate through claimed_amounts and claim_amount to verify that claim_amount is greater than/equal claimed_amount
    // Claimed_amounts are designed to be cumulative so that the user cannot replay
    // the same claim multiple times to get more rewards
    if user_info.claimed_amounts.is_empty() {
        user_info.claimed_amounts = vec![Uint128::zero(); claim_amounts.len()];
    }

    // let nfts_owned = state.unclaimed_amounts.iter().sum();

    for (i, amount) in claim_amounts.iter().enumerate() {
        if amount < &user_info.claimed_amounts[i] {
            return Err(StdError::generic_err(
                "claim amount cannot be smaller than the claimed amount",
            ));
        }
        let eligible_claim_amount = amount.checked_sub(user_info.claimed_amounts[i])?;
        let actual_claim_amount = state.unclaimed_amounts[i].min(eligible_claim_amount);

        state.unclaimed_amounts[i] = state.unclaimed_amounts[i].checked_sub(actual_claim_amount)?;

        user_info.claimed_amounts[i] =
            user_info.claimed_amounts[i].checked_add(actual_claim_amount)?;
        claimable_amounts.push(actual_claim_amount);
    }

    // save the new state
    STATE.save(deps.storage, &state)?;

    // Get sum of claimable amounts
    let claim_amount: Uint128 = claimable_amounts.iter().sum();

    // Transfer assets to the recipient
    let config = &CONFIG.load(deps.storage)?;

    // let seed_string = format!(
    //     "{},{},{},{},{}",
    //     config.asset.contract_addr.to_string(),
    //     recipient.to_string(),
    //     env.block.time.nanos(),
    //     0,
    //     // env.transaction.expect("expect transaction id").index,
    //     env.block.height
    // );

    let nft_ids = LIST_OF_IDS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .take(claim_amount.u128().try_into().unwrap())
        .map(|item| {
            let (key, _) = item?;
            Ok(key)
        })
        .collect::<StdResult<Vec<String>>>()?;

    // for each nft_id used up, remove from the map
    for nft_id in nft_ids.iter() {
        LIST_OF_IDS.remove(deps.storage, nft_id.to_string());
    }

    let messages = nft_ids
        .into_iter()
        .map(|nft_id| {
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.asset.contract_addr.to_string(),
                msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
                    recipient: recipient.to_string(),
                    token_id: nft_id.to_string(),
                })?,
                funds: vec![],
            });
            Ok(msg)
        })
        .collect::<StdResult<Vec<CosmosMsg>>>()?;

    let attributes = vec![
        attr("action", "genie_claim_rewards"),
        attr("receiver", recipient),
        attr("asset", config.asset.asset_string()),
        attr(
            "amount",
            user_info
                .claimed_amounts
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(","),
        ),
        attr(
            "receive_amount",
            claimable_amounts
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(","),
        ),
    ];

    USERS.save(deps.storage, recipient, &user_info)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(attributes))
}

fn query_state(deps: Deps, _env: Env) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    // let asset = CONFIG.load(deps.storage)?.asset;

    Ok(StateResponse {
        unclaimed_amounts: state.unclaimed_amounts,
    })
}

fn query_user_info(deps: Deps, user_address: String) -> StdResult<UserInfoResponse> {
    let user_address = deps.api.addr_validate(&user_address)?;
    let user_info = USERS
        .may_load(deps.storage, &user_address)?
        .unwrap_or_default();
    Ok(UserInfoResponse {
        claimed_amount: user_info.claimed_amounts,
    })
}

fn query_has_user_claimed(deps: Deps, user_address: String) -> StdResult<ClaimResponse> {
    let user_address = deps.api.addr_validate(&user_address)?;
    let user_info = USERS
        .may_load(deps.storage, &user_address)?
        .unwrap_or_default();
    Ok(ClaimResponse {
        has_claimed: !user_info.claimed_amounts.is_empty(),
    })
}

fn query_status(deps: Deps, env: &Env) -> StdResult<StatusResponse> {
    let config = CONFIG.load(deps.storage)?;
    let users_is_empty = USERS.is_empty(deps.storage);

    // check ownership by abusing pagination
    // query cw721 for tokens owned by this contract

    let page = 1;
    let limit = 1;
    let tokens: StdResult<cw721::TokensResponse> = deps.querier.query_wasm_smart(
        &config.asset.contract_addr,
        &cw721::Cw721QueryMsg::Tokens {
            owner: env.contract.address.to_string(),
            start_after: Some(page.to_string()),
            limit: Some(limit),
        },
    );

    let has_token = match tokens {
        Ok(tokens) => tokens.tokens.len() > 0,
        _ => false,
    };

    if env.block.time.seconds() < config.from_timestamp {
        Ok(StatusResponse {
            status: Status::NotStarted,
        })
    } else if env.block.time.seconds() >= config.from_timestamp && !has_token && users_is_empty {
        Ok(StatusResponse {
            status: Status::Invalid,
        })
    } else if env.block.time.seconds() >= config.from_timestamp
        && env.block.time.seconds() < config.to_timestamp
    {
        Ok(StatusResponse {
            status: Status::Ongoing,
        })
    } else {
        Ok(StatusResponse {
            status: Status::Ended,
        })
    }
}
