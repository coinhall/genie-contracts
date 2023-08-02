use crate::crypto::check_secp256k1_public_key;
use crate::state::{Config, CAMPAIGN_ADDRESSES, CONFIG};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
    Reply, ReplyOn, Response, StdError, StdResult, SubMsg, SubMsgResponse, SubMsgResult, Uint128,
    WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::{parse_instantiate_response_data, MsgInstantiateContractResponse};
use genie::airdrop::{QueryMsg as AirDropQueryMsg, Status as AirDropStatus, StatusResponse};
use genie::asset::AssetInfo;
use genie::factory::{
    AirdropInstantiateMsg, CampaignStatus, ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg,
    QueryMsg,
};

const CONTRACT_NAME: &str = "genie-airdrop-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// A `reply` call code ID used for sub-messages.
const INSTANTIATE_CONTRACT_REPLY_ID: u64 = 1;

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
            env,
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
    _env: Env,
    info: MessageInfo,
    asset_info: AssetInfo,
    from_timestamp: u64,
    to_timestamp: u64,
    allocated_amounts: Vec<Uint128>,
    campaign_id: String,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "genie_create_campaign"),
            attr("campaign_id", campaign_id),
        ])
        .add_submessage(SubMsg {
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: config.airdrop_code_id,
                funds: vec![],
                admin: None,
                label: String::from("Genie Campaign"),
                msg: to_binary(&AirdropInstantiateMsg {
                    owner: info.sender.to_string(),
                    asset: asset_info,
                    from_timestamp,
                    to_timestamp,
                    public_key: config.public_key.clone(),
                    allocated_amounts,
                })?,
            }),
            id: INSTANTIATE_CONTRACT_REPLY_ID,
            gas_limit: None,
            reply_on: ReplyOn::Success,
        }))
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
    let campaign_addrs_to_search = addresses
        .iter()
        .map(|address| {
            let is_valid = deps.api.addr_validate(&address);
            if is_valid.is_err() || !CAMPAIGN_ADDRESSES.has(deps.storage, &Addr::unchecked(address))
            {
                return Err(StdError::generic_err("Invalid address provided"));
            }
            Ok(Addr::unchecked(address))
        })
        .collect::<StdResult<Vec<_>>>()?;

    let statuses: StdResult<Vec<CampaignStatus>> = campaign_addrs_to_search
        .iter()
        .map(|addr| {
            let res = deps
                .querier
                .query_wasm_smart(addr, &AirDropQueryMsg::Status {})
                .unwrap_or(StatusResponse {
                    status: AirDropStatus::Invalid,
                });

            return Ok(CampaignStatus {
                address: addr.clone().into_string(),
                status: res.status,
            });
        })
        .collect();
    statuses
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Old genie contracts
    vec![
        "terra1ruzggfdthysya4330hh3gl3w05ex88v28d4gxl0pzwms7yntae6qaaafg9",
        "terra1cgez5kvmgmz0ukfz6dfdq04elye8nmyh0m6jmhgm8sd8jpd577wqyp0x77",
        "terra1grzz080a6lt9ygslfmnla9u5pyz8l20ar5d0e6nayr848cccyvxqgvc4xc",
        "terra19a3272fg55sj4hh6skz375crh2p4d9kralyu53r6e37p0uu020wsxq4qjh",
        "terra1g7cx8dmnkksg9rpjtusgd0c08vj695x35ryyrqx5h8c6yspumdeqzg8u8u",
        "terra1wkj7t03t5wqyqlxvnz3wl5jds3fy4v2t8macss8myxzd80h0ag8sl8zew0",
        "terra1jrf79q6l8jzlf72y7y96fmdx3d2humsklu7yjnxs4mdjfmg86vmq3qlmes",
        "terra1gzhd8zqenl3nfxqpnxlxc4q6c8850r8s8pyq30g90e4kq6fghvfqwq8lws",
        "terra1ajutl7paj8q7kwx2dhw6xnmdny6e0e4arkgsstzmxkthje3lwc6srlas6e",
        "terra1mlm3ftk4qgd9xrjdkfp9f3q7s0njqwsxanrvuw9ry0uwzed8xzgsaf5r7g",
        "terra1uqflfjztjpf5fkfq0ppgwwj329m9g35aef8uwtzy5z70u23pyywqdnjkzt",
        "terra1lxtxq8z6z9tsgd3e35mvtrhcr0yjc6jf786pw8fg7xrhl8l5phrq9hld0y",
        "terra1zw8fhk85sfzmk73n92q70qljnnap323g8g76peudeazzc0nw036q86ksxg",
        "terra19aufnajtmkavnjt6p99p8gl5na9xwwl7ahfmlrugkyrythc8v9fsjsh5ma",
        "terra13j7sj8fuhyjnkzjcz24t207v4nc8gs4xxdc3y3f98thddt3degvszl2wcc",
        "terra1mt59l2qkznq9g00u8nh0dlt7caaesavgcgp0z6ssq2h5kzsekzxsga60de",
        "terra1ccph3q35w4t0rdz3s0wmej840y6rw8szjmh3dywv5e29hlnllguqtc6p56",
        "terra1m090dkt73c4djjr8fqcmqvyd4ls4euujp8k5kcsu5ysa4gjqpswsyp3jvy",
        "terra1mlsxkekrzm4285zmyfdrz34ylala6fkauxcn6xv5newny00ye6nsgx5g3f",
        "terra1y57j0tz32dl3k9zxuzqfh54dljazl89dgnnreul5hzd48mwufanq5c64tl",
        "terra17z0gag5mcrnkggyrxvg3e8q9azj360qd9ksr6evaxlqx90lrvn5s7ufwy4",
        "terra1rfe2g7lzzxvgmfk5sjhqyke2g3465rgg5830hqnru0vkh0859dqq6zxdqz",
        "terra1pkyz3wdxs5xtyfmfgvxym45gancqw0wvznwusa3mrlj55epuhlzqh633v7",
        "terra15r3n0n2kv3dl8y4526rqy4acat8p06tlv30kf9nujylv3ppvjhmqllspa8",
        "terra1c5tkd93eg6e28plhpjyfv5zhptvqadz3lxgdz696myeh28ep3phs5nmr07",
        "terra12rftr93gwv9u55suefsl9ud9neymhkgh32s8d6z4n4uh7n6jp64qeu6qy2",
        "terra1crmm8d844l2zeazfx8upzgtyshvvvcjkphr79q66vfzhr492vmtsqjrpyq",
        "terra122r63zfahz0kydaau6n7atjz7nlen7vwh3ngdsvvnhtr53h5vlrsw3mzem",
        "terra1f9h342s6vh88camv0gu525m38hyqf035gfrx7lerhtd6an2u9cjsapvyuv",
        "terra16esxqu4f6pues6qs9x8kkghm2fgjpw9u49fyzt6zk43psdv98fys6dfhnn",
        "terra1xm9k0vq9f5rcf6qss70nlqgfejj3w65p5gw6ryhyy0spvpdln9qsg2y799",
        "terra1fnktcva8yw88w9zcgq93vu0vg70z4q5x0wt5yd3supjf4xdn9u0s0g9kfx",
        "terra1wsf30tndr5lfs8suh8rkfv2wucxaksm74rg8ezqpgwytk0up8nmqksssf6",
        "terra1sn87wp9sy6fukzczs0c99qdpjshhrredq6pf0lyfjpa0cenn8q7q65w4vw",
        "terra1vkmu9qq463jwgvhfxu3zg7du9dqx35szlavfnp28t462c0yddlysk40jdl",
        "terra1azq9ykezw52tyx3cuu25sc5y5ldvh8wldxrquf2qfgw6fa9zngfs053xs8",
        "terra17mkh5eygq546ua4p8qwsg6d088rr9aaad3mgz8fhtcgne4hxkvxqgmsv5t",
        "terra17xg7hcr4v8ec9pqxqkag28zx03pvj8r0le7aa5dyxzcql5fp4raqg5jxfd",
        "terra1vfrzah88yrmsg8lq6tfa57qdc77rvd0qdgzg7t8v09pppzupzrnstj3yjq",
        "terra1fqa03w5ekpunfezersx9urntcs05dggntazp4e8m634d8658usvq2v9z07",
        "terra1yzeule2wpyzacg7qgyrrtl922wz32tryz5cew55mcr34zdh0cruqyry4lj",
        "terra1v3v9cye3vxnvzvrcnz6vm7csfnf37p9xn2lgg4nq0r8pampzf0hq84r536",
        "terra1y7z3jsaxnyypavejc99udwsqs2yg2qykwczp7ku0l4t07mljk2pq9skwvc",
        "terra1pdzegx9kldlztr22yxh4mtn4quh78rm6vs2cky388fz8u39s5tesu9wthv",
        "terra1g89j0ghdqq88c5s5z97aptfemwmc54qe9a83k5hkzarfwmlf7x8q5dtya7",
        "terra16eq33svppq7mkx4x4atf92jx0qgrx8gxken7j8wrc8s8qw3src3snapfnt",
        "terra1cyf0v87dlfe42m7at6lwv5qpqk79p2z588xesmmkqyma82p0ug8sh6czrd",
        "terra1j0fdut6sgw3warxtsz7lq774keq4345yj9xnl33tnmd8gt58zj5q2rc83r",
    ]
    .iter()
    .for_each(|a| {
        let _ = CAMPAIGN_ADDRESSES.save(deps.storage, &Addr::unchecked(a.to_owned()), &Empty {});
    });

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg {
        Reply {
            id: _,
            result:
                SubMsgResult::Ok(SubMsgResponse {
                    data: Some(data), ..
                }),
        } => {
            let res: MsgInstantiateContractResponse = parse_instantiate_response_data(&data)
                .map_err(|_| StdError::generic_err("Error parsing instantiate reply"))?;

            CAMPAIGN_ADDRESSES.save(
                deps.storage,
                &Addr::unchecked(res.contract_address),
                &Empty {},
            )?;

            Ok(Response::default())
        }
        _ => Err(StdError::generic_err("Failed to parse reply")),
    }
}
