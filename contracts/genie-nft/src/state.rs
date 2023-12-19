use cosmwasm_std::{Addr, Binary, Uint128};
use cw_storage_plus::{Item, Map};
use genie::asset::NftInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// Account with 'owner' credentials
    pub owner: Addr,
    /// Campaign reward asset
    pub asset: NftInfo,
    /// Timestamp for the start of this campaign
    pub from_timestamp: u64,
    /// Timestamp for the end of this campaign
    pub to_timestamp: u64,
    /// Allocated amount of tokens for this campaign
    pub allocated_amounts: Vec<Uint128>,
    /// The public key used to verify claims
    pub public_key: Binary,
    /// The number of missions in this campaign
    pub mission_count: u64,
    /// The starting NFT id,
    pub start_id: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// Unclaimed amount, per mission, currently in this contract
    pub unclaimed_amounts: Vec<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct UserInfo {
    /// Assets claimed, per mission, by this account
    pub claimed_amounts: Vec<Uint128>,
    /// If applicable to this campaign, lootboxes claimed, per mission, by this account
    pub claimed_lootbox: Option<Vec<Uint128>>,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const USERS: Map<&Addr, UserInfo> = Map::new("users");

pub const LIST_OF_IDS: Map<u128, u128> = Map::new("list_of_ids");
