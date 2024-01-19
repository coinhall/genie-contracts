use crate::asset::NftInfo;
use cosmwasm_std::{Addr, Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub asset: NftInfo,
    pub public_key: Binary,
    pub from_timestamp: u64,
    pub to_timestamp: u64,
    pub allocated_amounts: Vec<Uint128>,
    pub start_id: Option<u128>,
    pub icon_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Claim {
        payload: Binary,
    },
    IncreaseIncentives {
        topup_amounts: Option<Vec<Uint128>>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    TransferUnclaimedTokens {
        recipient: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    IncreaseIncentives { topup_amounts: Option<Vec<Uint128>> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClaimNftPayload {
    pub claim_amounts: Vec<Uint128>,
    pub signature: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Status {},
    State {},
    UserInfo { address: String },
    HasUserClaimed { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
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
    /// The icon url of the NFT
    pub icon_url: String,
    /// The name of the NFT
    pub name: String,
    /// The symbol of the NFT
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfoResponse {
    pub claimed_amount: Vec<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimResponse {
    pub has_claimed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Status {
    NotStarted,
    Ongoing,
    Invalid,
    Ended,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StatusResponse {
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LastClaimerInfo {
    /// Assets claimed, per mission, by this account
    pub user_address: Addr,
    /// If applicable to this campaign, pending assets claimed, per mission, by this account
    pub pending_amount: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    /// Unclaimed amount, per mission, currently in this contract
    pub unclaimed_amounts: Vec<Uint128>,
    /// Current balance of the contract, used by UI to display the total amount of tokens held by this contract
    pub current_balance: Uint128,
}
