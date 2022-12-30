# Genie: Common Types

This is a collection of common types and the queriers which are commonly used in genie contracts.

## Data Types

### AssetInfo

AssetInfo is a convience wrapper to represent the native token and the contract token as a single type.

```rust
#[serde(rename_all = "snake_case")]
pub enum AssetInfo {
    Token { contract_addr: HumanAddr },
    NativeToken { denom: String },
}
```

## Functions

### Balance Querier

Query both native tokens and CW20 tokens using a single function

```rust
pub fn query_balance(
    querier: &QuerierWrapper,
    asset: &AssetInfo,
    account_addr: &Addr,
) -> StdResult<Uint128>
```

## Build transfer message

```rust
pub fn build_transfer_asset_msg(
    receipient: &Addr,
    asset: &AssetInfo,
    amount: Uint128,
) -> StdResult<CosmosMsg>
```
