use crate::crypto::is_valid_signature;
use crate::msg::{
    ClaimResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, Status, StatusResponse,
    UserInfoResponse,
};
use crate::state::{Config, State, CONFIG, STATE, USERS};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo,
    Order, Response, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use genie::asset::{build_transfer_asset_msg, AssetInfo};

const CONTRACT_NAME: &str = "genie";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    if msg.to_timestamp <= msg.from_timestamp {
        return Err(StdError::generic_err(
            "to_timestamp must be greater than from_timestamp",
        ));
    }

    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        asset: msg.asset,
        from_timestamp: msg.from_timestamp,
        to_timestamp: msg.to_timestamp,
        allocated_amount: msg.allocated_amount,
        public_key: msg.public_key,
    };
    CONFIG.save(deps.storage, &config)?;
    let state = State {
        unclaimed_amount: Uint128::zero(),
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
            claim_amount,
            signature,
        } => handle_claim(deps, env, info, claim_amount, signature),
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

    match config.asset {
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
            handle_increase_cw20_incentives(deps, env, cw20_msg.amount)
        }
    }
}

pub fn handle_increase_cw20_incentives(
    deps: DepsMut,
    env: Env,
    amount: Uint128,
) -> Result<Response, StdError> {
    if query_status(deps.as_ref(), &env)?.status != Status::NotStarted {
        return Err(StdError::generic_err(
            "rewards can only be deposited before campaign starts/ends",
        ));
    }
    let mut state = STATE.load(deps.storage)?;
    state.unclaimed_amount += amount;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "genie_increase_rewards")
        .add_attribute("current_reward_amount", state.unclaimed_amount))
}

pub fn handle_increase_native_incentives(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, StdError> {
    if query_status(deps.as_ref(), &env)?.status != Status::NotStarted {
        return Err(StdError::generic_err(
            "rewards can only be deposited before campaign starts/ends",
        ));
    }

    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(StdError::generic_err("can only be called by owner"));
    }

    let increase_amount: Uint128 = match config.asset {
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
    state.unclaimed_amount += increase_amount;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "genie_increase_rewards")
        .add_attribute("reward_size", state.unclaimed_amount))
}

pub fn handle_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    claim_amount: Uint128,
    signature: Binary,
) -> Result<Response, StdError> {
    if query_status(deps.as_ref(), &env)?.status != Status::Ongoing {
        return Err(StdError::generic_err("campaign is not ongoing"));
    }

    // Check if signature is valid
    let recipient = &info.sender;
    let is_valid = is_valid_signature(
        &deps,
        &recipient,
        &env.contract.address.to_string(),
        claim_amount,
        &signature,
        &CONFIG.load(deps.storage)?.public_key,
    )?;
    if !is_valid {
        return Err(StdError::generic_err("signature verification failed"));
    }
    // Check if recipient has already claimed the tokens
    let mut user_info = USERS.load(deps.storage, recipient).unwrap_or_default();
    if !user_info.claimed_amount.is_zero() {
        return Err(StdError::generic_err("address has already claimed once"));
    }
    // Check if rewards have already been fully claimed
    let mut state = STATE.load(deps.storage)?;
    if state.unclaimed_amount == Uint128::zero() {
        return Err(StdError::generic_err("rewards have been fully claimed"));
    }

    // Allow to claim remaining tokens if there are less tokens than the requested amount
    let claim_amount: Uint128 = state.unclaimed_amount.min(claim_amount);
    state.unclaimed_amount = state.unclaimed_amount.checked_sub(claim_amount)?;
    STATE.save(deps.storage, &state)?;
    user_info.claimed_amount = claim_amount;
    USERS.save(deps.storage, recipient, &user_info)?;

    // Transfer assets to the recipient
    let config = &CONFIG.load(deps.storage)?;
    let messages = vec![build_transfer_asset_msg(
        recipient,
        &config.asset,
        claim_amount,
    )?];

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "genie_claim_rewards"),
        attr("receiver", recipient),
        attr("asset", config.asset.asset_string()),
        attr("amount", claim_amount),
    ]))
}

pub fn handle_transfer_unclaimed_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount: Uint128,
) -> Result<Response, StdError> {
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
    let mut state = STATE.load(deps.storage)?;
    let amount = state.unclaimed_amount.min(amount);
    state.unclaimed_amount = state
        .unclaimed_amount
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
        airdrop_amount: user_info.claimed_amount,
    })
}

fn query_has_user_claimed(deps: Deps, user_address: String) -> StdResult<ClaimResponse> {
    let user_address = deps.api.addr_validate(&user_address)?;
    let user_info = USERS
        .may_load(deps.storage, &user_address)?
        .unwrap_or_default();
    Ok(ClaimResponse {
        is_claimed: !user_info.claimed_amount.is_zero(),
    })
}

fn query_status(deps: Deps, env: &Env) -> StdResult<StatusResponse> {
    let config = CONFIG.load(deps.storage)?;
    let users_count = USERS
        .range(deps.storage, None, None, Order::Ascending)
        .count();
    let state = STATE.load(deps.storage)?;

    if config.from_timestamp < env.block.time.seconds()
        && users_count == usize::from(0u8)
        && state.unclaimed_amount < config.allocated_amount
    {
        Ok(StatusResponse {
            status: Status::Invalid,
        })
    } else if env.block.time.seconds() < config.from_timestamp {
        Ok(StatusResponse {
            status: Status::NotStarted,
        })
    } else if env.block.time.seconds() > config.from_timestamp
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
