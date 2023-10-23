use crate::asset::AssetInfo;
use cosmwasm_std::{Addr, Binary, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub asset: AssetInfo,
    pub public_key: Binary,
    pub from_timestamp: u64,
    pub to_timestamp: u64,
    pub allocated_amounts: Vec<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Claim { payload: Binary },
    IncreaseIncentives { topup_amounts: Option<Vec<Uint128>> },
    TransferUnclaimedTokens { recipient: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    IncreaseIncentives { topup_amounts: Option<Vec<Uint128>> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ClaimPayload {
    pub claim_amounts: Vec<Uint128>,
    pub signature: Binary,
    pub lootbox_info: Option<Vec<Uint128>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Status {},
    State {},
    UserInfo { address: String },
    HasUserClaimed { address: String },
    UserLootboxInfo { address: String },
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
pub struct UserLootboxInfoResponse {
    pub claimed_lootbox: Vec<Uint128>,
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
    /// If applicable to this campaign, lootboxes claimed, per mission, by this account
    pub pending_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LastClaimerInfoWithMissionID {
    /// Assets claimed, per mission, by this account
    pub user_address: Addr,
    /// If applicable to this campaign, lootboxes claimed, per mission, by this account
    pub pending_amount: Uint128,
    /// Mission id
    pub mission_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    /// Unclaimed amount, per mission, currently in this contract
    pub unclaimed_amounts: Vec<Uint128>,
    /// Total funds protocol has sent and removed to this contract via `increase_incentives` and `transfer_unclaimed_tokens`
    pub protocol_funding: Uint128,
    /// Actual amount of tokens in this contract
    pub current_balance: Uint128,
    /// Last claimer info
    pub last_claimer_info: Vec<LastClaimerInfoWithMissionID>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LootboxInfo {
    pub claimed_lootbox: Binary,
}
