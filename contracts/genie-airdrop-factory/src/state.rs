use genie::factory::AirdropConfig;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Binary, Empty};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub public_key: Binary,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CAMPAIGN_ID_MAP: Map<String, Empty> = Map::new("campaign_id_map");
pub const AIRDROP_CONFIGS: Map<String, AirdropConfig> = Map::new("airdrop_configs");
