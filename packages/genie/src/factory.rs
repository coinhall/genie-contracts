use std::fmt::{Display, Formatter, Result};

use crate::{
    airdrop::Status,
    asset::{AssetInfo, NftInfo},
};
use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub public_key: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Update owner or public key
    UpdateConfig {
        owner: Option<String>,
        public_key: Option<Binary>,
    },
    /// Update relevant code IDs
    UpdateAirdropConfig { config: AirdropConfig },
    /// Create a new airdrop contract
    CreateAirdrop {
        asset_info: AssetInfo,
        from_timestamp: u64,
        to_timestamp: u64,
        allocated_amounts: Vec<Uint128>,
        campaign_id: String,
    },
    /// Create a new nft airdrop contract
    CreateNftAirdrop {
        nft_info: NftInfo,
        from_timestamp: u64,
        to_timestamp: u64,
        allocated_amounts: Vec<Uint128>,
        campaign_id: String,
        icon_url: String
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    CampaignStatuses { addresses: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AirdropConfig {
    pub airdrop_type: AirdropType,
    pub code_id: u64,
    pub is_disabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AirdropType {
    Asset,
    Nft,
}

/// Returns a raw encoded string representing the name of each airdrop type
impl Display for AirdropType {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self {
            AirdropType::Asset => fmt.write_str("asset"),
            AirdropType::Nft => fmt.write_str("nft"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub public_key: Binary,
    pub airdrop_configs: Vec<AirdropConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignStatus {
    pub address: String,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CampaignStatusesResponse {
    pub statuses: Vec<CampaignStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetAirdropInstantiateMsg {
    pub owner: String,
    pub asset: AssetInfo,
    pub public_key: Binary,
    pub from_timestamp: u64,
    pub to_timestamp: u64,
    pub allocated_amounts: Vec<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NftAirdropInstantiateMsg {
    pub owner: String,
    pub asset: NftInfo,
    pub public_key: Binary,
    pub from_timestamp: u64,
    pub to_timestamp: u64,
    pub allocated_amounts: Vec<Uint128>,
    pub icon_url: String
}
