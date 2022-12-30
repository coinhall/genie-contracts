use crate::crypto::verify_claim;
use crate::msg::{
    ClaimResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, Status, StatusResponse,
    UserInfoResponse,
};
use crate::state::{Config, State, CONFIG, PUBLIC_KEY, STATE, USERS};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo,
    Order, Response, StdError, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use genie::asset::{addr_validate_to_lower, build_transfer_asset_msg, query_balance, AssetInfo};

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
            "Invalid airdrop claim window closure timestamp",
        ));
    }

    let owner = addr_validate_to_lower(deps.api, msg.owner)?;

    let config = Config {
        owner,
        asset: msg.asset,
        from_timestamp: msg.from_timestamp,
        to_timestamp: msg.to_timestamp,
        allocated_amount: msg.allocated_amount,
    };

    let state = State {
        unclaimed_tokens: Uint128::zero(),
    };

    let public_key = msg.public_key;

    PUBLIC_KEY.save(deps.storage, &public_key)?;
    CONFIG.save(deps.storage, &config)?;
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
        QueryMsg::HasUserClaimed { address } => to_binary(&query_user_claimed(deps, address)?),
        QueryMsg::UserInfo { address } => to_binary(&query_user_info(deps, address)?),
        QueryMsg::Status {} => to_binary(&query_status(deps, &env)?),
        QueryMsg::State {} => to_binary(&STATE.load(deps.storage)?),
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
            return Err(StdError::generic_err("Invalid asset type"));
        }
        AssetInfo::Token { contract_addr } => {
            if info.sender != contract_addr {
                return Err(StdError::generic_err("Sender not authorized!"));
            }
        }
    };

    // CHECK :: CAN ONLY BE CALLED BY THE OWNER
    if cw20_msg.sender != config.owner {
        return Err(StdError::generic_err("Only owner can call this function!"));
    }

    // CHECK ::: Amount needs to be valid
    if cw20_msg.amount.is_zero() {
        return Err(StdError::generic_err("Amount must be greater than 0"));
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
            "Token deposit not allowed after airdrop start time!",
        ));
    }
    let mut state = STATE.load(deps.storage)?;

    state.unclaimed_tokens += amount;

    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("action", "airdrop incentives increased")
        .add_attribute("current_airdrop_size", state.unclaimed_tokens))
}

pub fn handle_increase_native_incentives(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, StdError> {
    if query_status(deps.as_ref(), &env)?.status != Status::NotStarted {
        return Err(StdError::generic_err(
            "Token deposit not allowed after airdrop start time!",
        ));
    }
    let mut state = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(StdError::generic_err("Sender not authorized!"));
    }

    let increase_amount: Uint128 = match config.asset {
        AssetInfo::NativeToken { denom } => info
            .funds
            .iter()
            .filter(|coin| coin.denom == denom)
            .map(|coin| coin.amount)
            .sum(),
        AssetInfo::Token { contract_addr: _ } => {
            return Err(StdError::generic_err("Invalid asset type"));
        }
    };

    if increase_amount.is_zero() {
        return Err(StdError::generic_err("Amount must be greater than 0"));
    }

    state.unclaimed_tokens += increase_amount;

    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("action", "airdrop incentives increased")
        .add_attribute("current_airdrop_size", state.unclaimed_tokens))
}

pub fn handle_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    claim_amount: Uint128,
    signature: Binary,
) -> Result<Response, StdError> {
    if query_status(deps.as_ref(), &env)?.status != Status::Ongoing {
        return Err(StdError::generic_err("Airdrop not ongoing"));
    }

    let asset = &CONFIG.load(deps.storage)?.asset;

    let receipient = &info.sender;
    let public_key = PUBLIC_KEY.load(deps.storage)?;
    if !verify_claim(
        &deps,
        &receipient,
        &env.contract.address.to_string(),
        claim_amount,
        &signature.0,
        &public_key.0,
    )? {
        return Err(StdError::generic_err("Signature verification failed"));
    }
    let mut user_info = USERS.load(deps.storage, receipient).unwrap_or_default();

    // Check if addr has already claimed the tokens
    if !user_info.airdrop_amount.is_zero() {
        return Err(StdError::generic_err("Already claimed"));
    }
    let unclaimed_tokens = STATE.load(deps.storage)?.unclaimed_tokens;
    if unclaimed_tokens == Uint128::zero() {
        return Err(StdError::generic_err(
            "Airdrop tokens have been fully claimed",
        ));
    }

    // Allow for claims to claim remaining tokens if there are less tokens than the claim amount
    let actual_amount: Uint128 = match unclaimed_tokens > claim_amount {
        true => claim_amount,
        false => unclaimed_tokens,
    };

    user_info.airdrop_amount = actual_amount;
    let updated_unclaimed_tokens = unclaimed_tokens.checked_sub(actual_amount)?;
    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.unclaimed_tokens = updated_unclaimed_tokens;
        Ok(state)
    })?;
    USERS.save(deps.storage, receipient, &user_info)?;

    // TRANSFER tokens to the user
    let mut messages = vec![];

    messages.push(build_transfer_asset_msg(receipient, &asset, actual_amount)?);

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        attr("action", "genie_claim"),
        attr("receiver", receipient),
        attr("asset", asset.asset_string()),
        attr("amount", actual_amount),
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

    // CHECK :: CAN ONLY BE CALLED BY THE OWNER
    if info.sender != config.owner {
        return Err(StdError::generic_err("Sender not authorized!"));
    }

    match query_status(deps.as_ref(), &env)?.status {
        Status::Ongoing => {
            return Err(StdError::generic_err(
                "Tokens cannot be transferred out while airdrop is ongoing",
            ));
        }
        _ => {}
    }

    let max_transferrable_tokens =
        query_balance(&deps.as_ref().querier, &config.asset, &env.contract.address)?;

    // CHECK :: Amount needs to be less than max_transferrable_tokens balance
    if amount > max_transferrable_tokens {
        return Err(StdError::generic_err(format!(
            "Amount cannot exceed max available token balance {}",
            max_transferrable_tokens
        )));
    }

    // UPDATE STATE
    let mut state = STATE.load(deps.storage)?;
    let updated_unclaimed_tokens = state
        .unclaimed_tokens
        .checked_sub(amount)
        .unwrap_or(Uint128::zero());
    state.unclaimed_tokens = updated_unclaimed_tokens;
    STATE.save(deps.storage, &state)?;

    // COSMOS MSG :: TRANSFER TOKENS
    let transfer_msg = build_transfer_asset_msg(&recipient, &config.asset, amount)?;

    Ok(Response::new()
        .add_message(transfer_msg)
        .add_attributes(vec![
            attr("action", "Airdrop::ExecuteMsg::TransferUnclaimedRewards"),
            attr("recipient", recipient),
            attr("amount", amount),
        ]))
}

fn query_user_info(deps: Deps, user_address: String) -> StdResult<UserInfoResponse> {
    let user_address = addr_validate_to_lower(deps.api, &user_address)?;
    let user_info = USERS
        .may_load(deps.storage, &user_address)?
        .unwrap_or_default();
    Ok(UserInfoResponse {
        airdrop_amount: user_info.airdrop_amount,
    })
}

fn query_user_claimed(deps: Deps, address: String) -> StdResult<ClaimResponse> {
    let user_address = addr_validate_to_lower(deps.api, &address)?;
    let user_info = USERS
        .may_load(deps.storage, &user_address)?
        .unwrap_or_default();

    Ok(ClaimResponse {
        is_claimed: !user_info.airdrop_amount.is_zero(),
    })
}

fn query_status(deps: Deps, env: &Env) -> StdResult<StatusResponse> {
    let config = CONFIG.load(deps.storage)?;
    let users_count = USERS
        .range(deps.storage, None, None, Order::Ascending)
        .count();
    let token_amount = STATE.load(deps.storage)?.unclaimed_tokens;

    if config.from_timestamp < env.block.time.seconds()
        && users_count == usize::from(0u8)
        && token_amount < config.allocated_amount
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
