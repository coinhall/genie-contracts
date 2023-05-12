use cosmwasm_std::{Addr, Binary, Uint128};
use cw_storage_plus::{Item, Map};
use genie::asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// Account with 'owner' credentials
    pub owner: Addr,
    /// Account with 'coinhall' credentials
    pub coinhall: Addr,
    /// Campaign reward asset
    pub asset: AssetInfo,
    /// Timestamp for the start of this campaign
    pub from_timestamp: u64,
    /// Timestamp for the end of this campaign
    pub to_timestamp: u64,
    /// Allocated amount of tokens for this campaign
    pub allocated_amount: Uint128,
    /// The public key used to verify claims
    pub public_key: Binary,
    /// The number of missions in this campaign
    pub mission_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// Unclaimed amount, per mission, currently in this contract
    pub unclaimed_amounts: Vec<Uint128>,
    /// Unclaimed amount for indexing and querying
    pub unclaimed_amount: Uint128,
    /// Premature end of campaign, coinhall and owner can end the campaign early if they do it together
    pub premature_end: bool,
    pub premature_end_timestamp: u64,
    pub premature_end_coinhall: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct UserInfo {
    /// Assets claimed, per mission, by this account
    pub claimed_amounts: Vec<Uint128>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const USERS: Map<&Addr, UserInfo> = Map::new("users");
