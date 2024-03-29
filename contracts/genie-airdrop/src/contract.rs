use crate::crypto::is_valid_signature;
use crate::state::{Config, State, CONFIG, LAST_CLAIMER, STATE, USERS};
use cosmwasm_std::{
    attr, entry_point, from_json, to_json_binary, Attribute, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Storage, Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use genie::airdrop::{
    ClaimPayload, ClaimResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, LastClaimerInfo,
    LastClaimerInfoWithMissionID, QueryMsg, StateResponse, Status, StatusResponse,
    UserInfoResponse, UserLootboxInfoResponse,
};
use genie::asset::{build_transfer_asset_msg, query_balance, AssetInfo};

const CONTRACT_NAME: &str = "genie-airdrop";
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
        ExecuteMsg::IncreaseIncentives { topup_amounts } => {
            receive_native(deps, env, info, topup_amounts)
        }
        ExecuteMsg::Claim { payload } => handle_claim(deps, env, info, payload),
        ExecuteMsg::TransferUnclaimedTokens { recipient } => {
            handle_transfer_unclaimed_tokens(deps, env, info, recipient)
        }
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
        QueryMsg::Status {} => to_json_binary(&query_status(deps.storage, &env)?),
        QueryMsg::UserLootboxInfo { address } => {
            to_json_binary(&query_user_lootbox_data(deps, address)?)
        }
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
    match from_json(&cw20_msg.msg)? {
        Cw20HookMsg::IncreaseIncentives { topup_amounts } => {
            handle_increase_incentives(deps, env, cw20_msg.amount, topup_amounts)
        }
    }
}

pub fn receive_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    topup_amounts: Option<Vec<Uint128>>,
) -> Result<Response, StdError> {
    let config = CONFIG.load(deps.storage)?;
    let amount: Uint128 = match config.asset.clone() {
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
    handle_increase_incentives(deps, env, amount, topup_amounts)
}

pub fn handle_increase_incentives(
    deps: DepsMut,
    env: Env,
    amount: Uint128,
    topup_amounts: Option<Vec<Uint128>>,
) -> Result<Response, StdError> {
    if amount.is_zero() {
        return Err(StdError::generic_err("amount must be greater than 0"));
    }

    let storage = deps.storage;
    let (msgs, attributes) = match query_status(storage, &env)? {
        StatusResponse {
            status: Status::Ongoing,
        } => {
            if topup_amounts.is_none() {
                return Err(StdError::generic_err(
                    "topup_amounts must be present during an ongoing campaign",
                ));
            }
            handle_topup(storage, amount, topup_amounts.unwrap())?
        }
        StatusResponse {
            status: Status::NotStarted,
        } => {
            if topup_amounts.is_some() {
                return Err(StdError::generic_err(
                    "topup_amounts must not be present before campaign starts",
                ));
            }
            (vec![], vec![])
        }
        StatusResponse { status } => {
            return Err(StdError::generic_err(format!(
                "increasing incentives is not allowed in {:?} status",
                status
            )));
        }
    };

    let config = CONFIG.load(storage)?;
    let mut state = STATE.load(storage)?;
    state.protocol_funding += amount;
    STATE.save(storage, &state)?;

    Ok(Response::new()
        .add_messages(msgs)
        .add_attributes(vec![
            attr("action", "genie_increase_incentives"),
            attr("asset", config.asset.asset_string()),
            attr("protocol_funding", state.protocol_funding),
            attr("increase_amount", amount),
        ])
        .add_attributes(attributes))
}

pub fn handle_topup(
    storage: &mut dyn Storage,
    amount: Uint128,
    topup_amounts: Vec<Uint128>,
) -> Result<(Vec<CosmosMsg>, Vec<Attribute>), StdError> {
    let mut state = STATE.load(storage)?;
    let mut config = CONFIG.load(storage)?;
    if topup_amounts.len() != state.unclaimed_amounts.len() {
        return Err(StdError::generic_err(
            "topup amount length does not match claimable amount length",
        ));
    }

    let sum = topup_amounts.iter().sum::<Uint128>();
    if amount != sum {
        return Err(StdError::generic_err(
            "topup amount does not match the amount sent",
        ));
    }

    let mut msgs: Vec<CosmosMsg> = vec![];
    let mut attributes: Vec<Attribute> = vec![
        attr(
            "topup_amounts",
            topup_amounts
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(","),
        ),
        attr(
            "unclaimed_amounts",
            state
                .unclaimed_amounts
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(","),
        ),
    ];
    for (i, topup_amount) in topup_amounts.iter().enumerate() {
        config.allocated_amounts[i] = config.allocated_amounts[i].checked_add(*topup_amount)?;
        state.unclaimed_amounts[i] = state.unclaimed_amounts[i].checked_add(*topup_amount)?;

        // Check for last claimer activity
        if !topup_amount.is_zero() {
            if let Ok(last_claimer) = LAST_CLAIMER.load(storage, i as u64) {
                let claim_amount = last_claimer.pending_amount.min(*topup_amount);
                if last_claimer.pending_amount == claim_amount {
                    LAST_CLAIMER.remove(storage, i as u64);
                } else {
                    LAST_CLAIMER.save(
                        storage,
                        i as u64,
                        &LastClaimerInfo {
                            user_address: last_claimer.user_address.clone(),
                            pending_amount: last_claimer
                                .pending_amount
                                .checked_sub(claim_amount)?,
                        },
                    )?;
                }
                let mut user = USERS.load(storage, &last_claimer.user_address)?;
                user.claimed_amounts[i] = user.claimed_amounts[i].checked_add(claim_amount)?;
                USERS.save(storage, &last_claimer.user_address, &user)?;

                msgs.push(build_transfer_asset_msg(
                    &last_claimer.user_address,
                    &config.asset,
                    claim_amount,
                )?);

                attributes.extend(vec![
                    attr("action", "genie_pending_claim"),
                    attr("asset", config.asset.asset_string()),
                    attr("amount", claim_amount),
                    attr("receiver", last_claimer.user_address),
                    attr("mission_id", i.to_string()),
                ]);
                state.unclaimed_amounts[i] =
                    state.unclaimed_amounts[i].checked_sub(claim_amount)?;
            }
        }
    }

    STATE.save(storage, &state)?;
    CONFIG.save(storage, &config)?;

    Ok((msgs, attributes))
}

pub fn handle_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    payload: Binary,
) -> Result<Response, StdError> {
    let ClaimPayload {
        claim_amounts,
        signature,
        lootbox_info,
    } = from_json(&payload)?;
    if query_status(deps.storage, &env)?.status != Status::Ongoing {
        return Err(StdError::generic_err("campaign is not ongoing"));
    }

    let recipient = &info.sender;
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
        let eligible_claim_amount = amount.checked_sub(user_info.claimed_amounts[i])?;
        let actual_claim_amount = state.unclaimed_amounts[i].min(eligible_claim_amount);

        state.unclaimed_amounts[i] = state.unclaimed_amounts[i].checked_sub(actual_claim_amount)?;

        // check for last claimer eligibility
        if state.unclaimed_amounts[i] == Uint128::zero()
            && eligible_claim_amount > actual_claim_amount
            && actual_claim_amount > Uint128::zero()
        {
            LAST_CLAIMER.save(
                deps.storage,
                i as u64,
                &LastClaimerInfo {
                    user_address: recipient.clone(),
                    pending_amount: eligible_claim_amount.checked_sub(actual_claim_amount)?,
                },
            )?;
        }

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
    let messages = if claim_amount == Uint128::zero() {
        vec![]
    } else {
        vec![build_transfer_asset_msg(
            recipient,
            &config.asset,
            claim_amount,
        )?]
    };

    let mut attributes = vec![
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

    // Lootbox specific logic
    // Convert lootbox_amounts to string
    if let Some(lootbox_info) = lootbox_info {
        // Vec length must coincide with mission length
        if lootbox_info.len() != claim_amounts.len() {
            return Err(StdError::generic_err(
                "lootbox amounts length does not match claimable amount length",
            ));
        }
        let lootbox_amounts_string = lootbox_info
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");

        user_info.claimed_lootbox = Some(lootbox_info);
        attributes.push(attr("claimed_lootbox", lootbox_amounts_string))
    }

    USERS.save(deps.storage, recipient, &user_info)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attributes(attributes))
}

pub fn handle_transfer_unclaimed_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
) -> Result<Response, StdError> {
    let recipient = deps.api.addr_validate(&recipient)?;
    let config = CONFIG.load(deps.storage)?;

    // Can only be called by owner
    if info.sender != config.owner {
        return Err(StdError::generic_err("can only be called by owner"));
    }
    // Can only withdraw if campaign is not ongoing
    let status = query_status(deps.storage, &env)?.status;
    if status == Status::Ongoing {
        return Err(StdError::generic_err(
            "cannot withdraw while campaign is ongoing",
        ));
    }

    // Balance in this contract must be queried to handle the case where assets was deposited
    // without using the `increase_incentives` execute msg.
    let amount = query_balance(&deps.as_ref().querier, &env.contract.address, &config.asset)?;
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

fn query_state(deps: Deps, env: Env) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    let asset = CONFIG.load(deps.storage)?.asset;
    let current_balance = query_balance(&deps.querier, &env.contract.address, &asset)?;
    // LOAD all the last_claimers
    let last_claimers = LAST_CLAIMER
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (mission_id, last_claimer) = item?;
            Ok(LastClaimerInfoWithMissionID {
                user_address: last_claimer.user_address,
                pending_amount: last_claimer.pending_amount,
                mission_id,
            })
        })
        .collect::<StdResult<Vec<LastClaimerInfoWithMissionID>>>()?;

    Ok(StateResponse {
        unclaimed_amounts: state.unclaimed_amounts,
        protocol_funding: state.protocol_funding,
        current_balance,
        last_claimer_info: last_claimers,
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

fn query_status(storage: &dyn Storage, env: &Env) -> StdResult<StatusResponse> {
    let config = CONFIG.load(storage)?;
    let users_is_empty = USERS.is_empty(storage);
    let state = STATE.load(storage)?;
    let current_amount = state.protocol_funding;

    if env.block.time.seconds() >= config.from_timestamp
        && users_is_empty
        && current_amount < config.allocated_amounts.iter().sum()
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

fn query_user_lootbox_data(deps: Deps, user_address: String) -> StdResult<UserLootboxInfoResponse> {
    let user_address = deps.api.addr_validate(&user_address)?;
    let user_info = USERS
        .may_load(deps.storage, &user_address)?
        .unwrap_or_default();
    let claimed_lootbox = if let Some(claimed_lootbox) = user_info.claimed_lootbox {
        claimed_lootbox
    } else {
        Vec::new()
    };
    Ok(UserLootboxInfoResponse { claimed_lootbox })
}
