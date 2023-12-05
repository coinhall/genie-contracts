use cosmwasm_std::{
    to_json_binary, Addr, AllBalanceResponse, BankMsg, BankQuery, Coin, CosmosMsg, QuerierWrapper,
    QueryRequest, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A wrapper to represent both native coins and cw20 tokens as a single type
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetInfo {
    Token { contract_addr: Addr },
    NativeToken { denom: String },
}

/// A wrapper to represent both native coins and cw20 tokens as a single type
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct NftInfo {
    pub contract_addr: Addr,
}

impl NftInfo {
    pub fn asset_string(&self) -> String {
        self.contract_addr.to_string()
    }
}

impl AssetInfo {
    pub fn asset_string(&self) -> String {
        match self {
            AssetInfo::Token { contract_addr } => contract_addr.to_string(),
            AssetInfo::NativeToken { denom } => denom.to_string(),
        }
    }
}

/// Queries the balance of `asset` in `account_addr`
pub fn query_balance(
    querier: &QuerierWrapper,
    account_addr: &Addr,
    asset: &AssetInfo,
) -> StdResult<Uint128> {
    match asset {
        AssetInfo::Token { contract_addr } => {
            get_cw20_balance(querier, account_addr, contract_addr)
        }
        AssetInfo::NativeToken { denom } => get_native_balance(querier, account_addr, denom),
    }
}

fn get_native_balance(
    querier: &QuerierWrapper,
    account_addr: &Addr,
    denom: &String,
) -> StdResult<Uint128> {
    let query = QueryRequest::Bank(BankQuery::AllBalances {
        address: account_addr.into(),
    });
    let balances: AllBalanceResponse = querier.query(&query)?;
    Ok(balances
        .amount
        .into_iter()
        .find(|balance| balance.denom.eq(denom))
        .map(|balance| balance.amount)
        .unwrap_or_default())
}

fn get_cw20_balance(
    querier: &QuerierWrapper,
    account_addr: &Addr,
    token_address: &Addr,
) -> StdResult<Uint128> {
    let query: BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token_address.into(),
        msg: to_json_binary(&Cw20QueryMsg::Balance {
            address: account_addr.into(),
        })?,
    }))?;

    Ok(query.balance)
}

/// Builds the transfer message for a given `asset`
pub fn build_transfer_asset_msg(
    recipient: &Addr,
    asset: &AssetInfo,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    match asset {
        AssetInfo::Token { contract_addr } => {
            build_transfer_cw20_token_msg(recipient, contract_addr.into(), amount)
        }
        AssetInfo::NativeToken { denom } => {
            build_transfer_native_token_msg(recipient, denom.into(), amount)
        }
    }
}

fn build_transfer_native_token_msg(
    recipient: &Addr,
    denom: String,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![Coin { denom, amount }],
    }))
}

fn build_transfer_cw20_token_msg(
    recipient: &Addr,
    token_contract_address: String,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_contract_address,
        msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.into(),
            amount,
        })?,
        funds: vec![],
    }))
}
