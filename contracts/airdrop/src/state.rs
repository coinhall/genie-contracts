use cosmwasm_std::{Addr, Binary, Uint128};
use cw_storage_plus::{Item, Map};
use genie::asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// Protocol account who can update config
    pub owner: Addr,
    ///  Airdrop token address
    pub asset: AssetInfo,
    /// Timestamp since which airdrops can be delegated to bootstrap auction contract
    pub from_timestamp: u64,
    /// Timestamp to which airdrops can be claimed
    pub to_timestamp: u64,
    /// Allocated amount of tokens for airdrop
    pub allocated_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub unclaimed_tokens: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct UserInfo {
    /// Total airdrop tokens claimed by the user
    pub airdrop_amount: Uint128,
}

/// Stores the config struct at the given key
pub const CONFIG: Item<Config> = Item::new("config");
/// Stores user information. Key is address, value is the user info struct
pub const USERS: Map<&Addr, UserInfo> = Map::new("users");

pub const PUBLIC_KEY: Item<Binary> = Item::new("public_key");

pub const STATE: Item<State> = Item::new("state");
