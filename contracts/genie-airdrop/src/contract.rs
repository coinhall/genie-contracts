use crate::crypto::is_valid_signature;
use crate::state::{Config, State, CONFIG, STATE, USERS};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use genie::airdrop::{
    ClaimResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, Status, StatusResponse,
    UserInfoResponse,
};
use genie::asset::{build_transfer_asset_msg, query_balance, AssetInfo};

const CONTRACT_NAME: &str = "genie-airdrop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    if msg.from_timestamp <= _env.block.time.seconds() {
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
        allocated_amount: msg.allocated_amounts.iter().sum(),
        public_key: msg.public_key,
        mission_count: msg.allocated_amounts.len() as u64,
    };
    CONFIG.save(deps.storage, &config)?;
    let state = State {
        unclaimed_amounts: msg.allocated_amounts,
        protocol_funding: Uint128::zero(),
    };
    STATE.save(deps.storage, &state)?;

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
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::IncreaseIncentives {} => handle_increase_native_incentives(deps, env, info),
        ExecuteMsg::Claim {
            claim_amounts,
            signature,
        } => handle_claim(deps, env, info, claim_amounts, signature),
        ExecuteMsg::TransferUnclaimedTokens { recipient, amount } => {
            handle_transfer_unclaimed_tokens(deps, env, info, recipient, amount)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::State {} => to_binary(&STATE.load(deps.storage)?),
        QueryMsg::HasUserClaimed { address } => to_binary(&query_has_user_claimed(deps, address)?),
        QueryMsg::UserInfo { address } => to_binary(&query_user_info(deps, address)?),
        QueryMsg::Status {} => to_binary(&query_status(deps, &env)?),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, StdError> {
    let config = CONFIG.load(deps.storage)?;

    match config.asset.clone() {
        AssetInfo::NativeToken { denom: _ } => {
            return Err(StdError::generic_err("invalid asset type"));
        }
        AssetInfo::Token { contract_addr } => {
            if info.sender != contract_addr {
                return Err(StdError::generic_err(
                    "can only be called by token contract",
                ));
            }
        }
    };

    if cw20_msg.sender != config.owner {
        return Err(StdError::generic_err("can only be called by owner"));
    }
    if cw20_msg.amount.is_zero() {
        return Err(StdError::generic_err("amount must be greater than 0"));
    }

    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::IncreaseIncentives {} => {
            handle_increase_cw20_incentives(deps, env, config.asset.asset_string(), cw20_msg.amount)
        }
    }
}

pub fn handle_increase_cw20_incentives(
    deps: DepsMut,
    env: Env,
    asset: String,
    amount: Uint128,
) -> Result<Response, StdError> {
    if query_status(deps.as_ref(), &env)?.status != Status::NotStarted {
        return Err(StdError::generic_err(
            "rewards can only be deposited before campaign starts",
        ));
    }
    let mut state = STATE.load(deps.storage)?;
    state.protocol_funding += amount;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "genie_increase_rewards"),
        attr("asset", asset),
        attr("protocol_funding", state.protocol_funding),
    ]))
}

pub fn handle_increase_native_incentives(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, StdError> {
    if query_status(deps.as_ref(), &env)?.status != Status::NotStarted {
        return Err(StdError::generic_err(
            "rewards can only be deposited before campaign starts",
        ));
    }

    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("can only be called by owner"));
    }

    let increase_amount: Uint128 = match config.asset.clone() {
        AssetInfo::NativeToken { denom } => info
            .funds
            .iter()
            .filter(|coin| coin.denom == denom)
            .map(|coin| coin.amount)
            .sum(),
        AssetInfo::Token { contract_addr: _ } => {
            return Err(StdError::generic_err("invalid asset type"));
        }
    };
    if increase_amount.is_zero() {
        return Err(StdError::generic_err("amount must be greater than 0"));
    }

    let mut state = STATE.load(deps.storage)?;
    state.protocol_funding += increase_amount;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "genie_increase_rewards"),
        attr("asset", config.asset.asset_string()),
        attr("protocol_funding", state.protocol_funding),
    ]))
}

pub fn handle_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    claim_amounts: Binary,
    signature: Binary,
) -> Result<Response, StdError> {
    if query_status(deps.as_ref(), &env)?.status != Status::Ongoing {
        return Err(StdError::generic_err("campaign is not ongoing"));
    }

    // convert claim_amounts to string
    let claim_string = String::from_utf8(claim_amounts.to_vec())?;
    let claim_amounts = claim_string
        .split(',')
        .map(|x| x.parse::<Uint128>())
        .collect::<Result<Vec<Uint128>, _>>()?;

    let recipient = &info.sender;
    let mut user_info = USERS.load(deps.storage, recipient).unwrap_or_default();
    let mut state = STATE.load(deps.storage)?;

    if claim_amounts.len() != state.unclaimed_amounts.len() {
        return Err(StdError::generic_err(
            "claim amount length does not match claimable amount length",
        ));
    }

    // Check if signature is valid
    let is_valid = is_valid_signature(
        &deps,
        recipient,
        &env.contract.address.to_string(),
        claim_amounts.clone(),
        &signature,
        &CONFIG.load(deps.storage)?.public_key,
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

    for (i, amount) in claim_amounts.iter().enumerate() {
        if amount < &user_info.claimed_amounts[i] {
            return Err(StdError::generic_err(
                "claim amount cannot be smaller than the claimed amount",
            ));
        }
        let difference = amount.checked_sub(user_info.claimed_amounts[i])?;
        let actual_claim_amount = state.unclaimed_amounts[i].min(difference);
        state.unclaimed_amounts[i] = state.unclaimed_amounts[i].checked_sub(actual_claim_amount)?;
        user_info.claimed_amounts[i] =
            user_info.claimed_amounts[i].checked_add(actual_claim_amount)?;
        claimable_amounts.push(actual_claim_amount);
    }

    USERS.save(deps.storage, recipient, &user_info)?;
    // save the new state
    STATE.save(deps.storage, &state)?;

    // Get sum of claimable amounts
    let claim_amount: Uint128 = claimable_amounts.iter().sum();

    // Transfer assets to the recipient
    let config = &CONFIG.load(deps.storage)?;
    let messages = if claim_amount == Uint128::zero() {
        vec![]
    } else {
        vec![build_transfer_asset_msg(
            recipient,
            &config.asset,
            claim_amount,
        )?]
    };

    Ok(Response::new().add_messages(messages).add_attributes(vec![
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
    ]))
}

pub fn handle_transfer_unclaimed_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, StdError> {
    let recipient = deps.api.addr_validate(&recipient)?;
    let config = CONFIG.load(deps.storage)?;

    // Can only be called by owner
    if info.sender != config.owner {
        return Err(StdError::generic_err("can only be called by owner"));
    }
    // Can only withdraw if campaign is not ongoing
    if query_status(deps.as_ref(), &env)?.status == Status::Ongoing {
        return Err(StdError::generic_err(
            "cannot withdraw while campaign is ongoing",
        ));
    }

    // Allow transfers of remaining tokens if there are less tokens than the requested amount
    // Balance in this contract must be queried to handle the case where assets was deposited
    // without using the `increase_incentives` execute msg.
    let max_transferable_amount =
        query_balance(&deps.as_ref().querier, &env.contract.address, &config.asset)?;
    let amount = max_transferable_amount.min(amount);
    let mut state = STATE.load(deps.storage)?;
    state.protocol_funding = state
        .protocol_funding
        .checked_sub(amount)
        .unwrap_or(Uint128::zero());
    STATE.save(deps.storage, &state)?;

    // Transfer assets to recipient
    let transfer_msg = build_transfer_asset_msg(&recipient, &config.asset, amount)?;

    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attributes(vec![
            attr("action", "genie_transfer_unclaimed_rewards"),
            attr("receiver", recipient),
            attr("asset", config.asset.asset_string()),
            attr("amount", amount),
        ]))
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
    let state = STATE.load(deps.storage)?;
    let current_amount = state.protocol_funding;

    if env.block.time.seconds() >= config.from_timestamp
        && users_is_empty
        && current_amount < config.allocated_amount
    {
        Ok(StatusResponse {
            status: Status::Invalid,
        })
    } else if env.block.time.seconds() < config.from_timestamp {
        Ok(StatusResponse {
            status: Status::NotStarted,
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
