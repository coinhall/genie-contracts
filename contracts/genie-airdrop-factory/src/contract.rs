use crate::crypto::check_secp256k1_public_key;
use crate::state::{Config, CONFIG};
use cosmwasm_std::{
    attr, entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use genie::airdrop::{QueryMsg as AirdropQueryMsg, StatusResponse};
use genie::asset::AssetInfo;
use genie::factory::{
    AirdropInstantiateMsg, CampaignStatus, ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg,
    QueryMsg,
};

const CONTRACT_NAME: &str = "genie-airdrop-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if !check_secp256k1_public_key(&msg.public_key) {
        return Err(StdError::generic_err("Invalid public key"));
    }

    let config = Config {
        owner: info.sender,
        airdrop_code_id: msg.airdrop_code_id,
        public_key: msg.public_key,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            airdrop_code_id,
            public_key,
        } => execute_update_config(deps, env, info, owner, airdrop_code_id, public_key),
        ExecuteMsg::CreateAirdrop {
            asset_info,
            from_timestamp,
            to_timestamp,
            allocated_amounts,
            campaign_id,
        } => execute_create_airdrop(
            deps,
            info,
            asset_info,
            from_timestamp,
            to_timestamp,
            allocated_amounts,
            campaign_id,
        ),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
    airdrop_code_id: Option<u64>,
    public_key: Option<Binary>,
) -> StdResult<Response> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(StdError::generic_err("can only be called by owner"));
    }
    if let Some(owner) = owner {
        // validate address format
        config.owner = deps.api.addr_validate(&owner)?;
    }
    if let Some(airdrop_code_id) = airdrop_code_id {
        config.airdrop_code_id = airdrop_code_id;
    }
    if let Some(public_key) = public_key {
        if !check_secp256k1_public_key(&public_key) {
            return Err(StdError::generic_err("invalid public key"));
        }
        config.public_key = public_key;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "genie_update_config"))
}

pub fn execute_create_airdrop(
    deps: DepsMut,
    info: MessageInfo,
    asset_info: AssetInfo,
    from_timestamp: u64,
    to_timestamp: u64,
    allocated_amounts: Vec<Uint128>,
    campaign_id: String,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    if allocated_amounts.len() > 5 {
        return Err(StdError::generic_err("too many allocation amounts"));
    }

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "genie_create_campaign"),
            attr("campaign_id", campaign_id),
        ])
        .add_message(CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: config.airdrop_code_id,
            funds: vec![],
            admin: None,
            label: String::from("Genie Campaign"),
            msg: to_binary(&AirdropInstantiateMsg {
                owner: info.sender.to_string(),
                asset: asset_info,
                from_timestamp,
                to_timestamp,
                public_key: config.public_key,
                allocated_amounts,
            })?,
        })))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::CampaignStatuses { addresses } => {
            to_binary(&query_campaign_statuses(deps, addresses)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let res = ConfigResponse {
        owner: state.owner.to_string(),
        airdrop_code_id: state.airdrop_code_id,
        public_key: state.public_key,
    };
    Ok(res)
}

pub fn query_campaign_statuses(
    deps: Deps,
    addresses: Vec<String>,
) -> StdResult<Vec<CampaignStatus>> {
    addresses
        .iter()
        .map(|addr| {
            let res: StatusResponse = deps
                .querier
                .query_wasm_smart(addr, &AirdropQueryMsg::Status {})?;

            Ok(CampaignStatus {
                address: addr.into(),
                status: res.status,
            })
        })
        .collect()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
