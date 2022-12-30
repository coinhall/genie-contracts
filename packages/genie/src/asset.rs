use cosmwasm_std::{
    to_binary, Addr, AllBalanceResponse, Api, BankMsg, BankQuery, Coin, CosmosMsg, QuerierWrapper,
    QueryRequest, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Returns a lowercased, validated address upon success.
pub fn addr_validate_to_lower(api: &dyn Api, addr: impl Into<String>) -> StdResult<Addr> {
    let addr = addr.into();
    if addr.to_lowercase() != addr {
        return Err(StdError::generic_err(format!(
            "Address {} should be lowercase",
            addr
        )));
    }
    api.addr_validate(&addr)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetInfo {
    Token { contract_addr: Addr },
    NativeToken { denom: String },
}

impl AssetInfo {
    pub fn asset_string(&self) -> String {
        match self {
            AssetInfo::Token { contract_addr } => contract_addr.to_string(),
            AssetInfo::NativeToken { denom } => denom.to_string(),
        }
    }
}

pub fn query_balance(
    querier: &QuerierWrapper,
    asset: &AssetInfo,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    match asset {
        AssetInfo::Token { contract_addr } => {
            cw20_get_balance(querier, contract_addr, account_addr)
        }
        AssetInfo::NativeToken { denom } => native_get_balance(querier, account_addr, denom),
    }
}

pub fn build_transfer_asset_msg(
    receipient: &Addr,
    asset: &AssetInfo,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    match asset {
        AssetInfo::Token { contract_addr } => {
            build_transfer_cw20_token_msg(receipient, &contract_addr.to_string(), amount)
        }
        AssetInfo::NativeToken { denom } => {
            build_transfer_native_token_msg(receipient, &denom, amount)
        }
    }
}

pub fn build_transfer_native_token_msg(
    recipient: &Addr,
    denom: &String,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![Coin {
            denom: denom.clone(),
            amount,
        }],
    }))
}

pub fn build_transfer_cw20_token_msg(
    recipient: &Addr,
    token_contract_address: &String,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_contract_address.clone(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.into(),
            amount,
        })?,
        funds: vec![],
    }))
}

pub fn native_get_balance(
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

pub fn cw20_get_balance(
    querier: &QuerierWrapper,
    token_address: &Addr,
    account_addr: &Addr,
) -> StdResult<Uint128> {
    let query: BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token_address.into(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: account_addr.into(),
        })?,
    }))?;

    Ok(query.balance)
}
