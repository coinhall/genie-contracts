use std::convert::TryInto;
use std::vec;

use crate::crypto::is_valid_signature;
use crate::state::{Config, State, CONFIG, LIST_OF_IDS, STATE, USERS};
use cosmwasm_std::{
    attr, entry_point, from_json, to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use genie::airdrop_nft::{
    ClaimNftPayload, ClaimResponse, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse, Status,
    StatusResponse, UserInfoResponse,
};
use rand::Rng;
use rand_core::SeedableRng;
use rand_xoshiro::Xoshiro128PlusPlus;
use sha3::{Digest, Keccak256};
use shuffle::{fy::FisherYates, shuffler::Shuffler};

const CONTRACT_NAME: &str = "genie-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
    let mut vec_of_ids: Vec<String> = vec![];

    for i in 0..end_id.u128() {
        let id = i.to_string();
        vec_of_ids.push(id);
    }

    let seed_string = format!(
        "{},{},{},{}",
        config.asset.contract_addr.to_string(),
        info.sender.to_string(),
        env.block.time.nanos(),
        env.block.height
    );
    let randomized_ids = shuffle_vector(seed_string, vec_of_ids)?;

    for (i, id) in randomized_ids.iter().enumerate() {
        LIST_OF_IDS.save(deps.storage, i as u128, id)?;
    }

    Ok(Response::default())
}

fn shuffle_vector(seed: String, mut vector: Vec<String>) -> StdResult<Vec<String>> {
    let hash = Keccak256::digest(seed.as_bytes());
    let randomness: [u8; 16] = hash.to_vec()[0..16].try_into().unwrap();
    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);
    let mut shuffler = FisherYates::default();
    shuffler
        .shuffle(&mut vector, &mut rng)
        .map_err(StdError::generic_err)?;

    Ok(vector)
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

    let mut last = state
        .unclaimed_amounts
        .clone()
        .into_iter()
        .sum::<Uint128>()
        .u128()
        - 1;

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

    let seed_string = format!(
        "{},{},{},{}",
        config.asset.contract_addr.to_string(),
        info.sender.to_string(),
        env.block.time.nanos(),
        env.block.height
    );
    let hash = Keccak256::digest(seed_string.as_bytes());
    let randomness: [u8; 16] = hash.to_vec()[0..16].try_into().unwrap();
    let mut rng = Xoshiro128PlusPlus::from_seed(randomness);
    let rngesus: &mut dyn rand::RngCore = &mut rng;

    let mut nft_ids: Vec<String> = vec![];
    for _ in 0..claim_amount.u128() {
        let random_number = rngesus.gen_range(0..=last);
        let id = LIST_OF_IDS
            .load(deps.storage, random_number)
            .unwrap_or_default();
        let last_id = LIST_OF_IDS.load(deps.storage, last)?;
        LIST_OF_IDS.save(deps.storage, random_number, &last_id)?;
        LIST_OF_IDS.remove(deps.storage, last);
        last = last.checked_sub(1).unwrap_or_default();
        nft_ids.push(id);
    }

    let messages = nft_ids
        .into_iter()
        .map(|nft_id| {
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.asset.contract_addr.to_string(),
                msg: to_json_binary(&cw721::Cw721ExecuteMsg::TransferNft {
                    recipient: recipient.to_string(),
                    token_id: nft_id,
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

    // check ownership by using pagination
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
        Ok(tokens) => !tokens.tokens.is_empty(),
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
