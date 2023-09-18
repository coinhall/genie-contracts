use cosmwasm_std::{Addr, DepsMut, StdResult, Uint128};
use sha3::{Digest, Keccak256};

pub fn is_valid_signature(
    deps: &DepsMut,
    account: &Addr,
    asset_string: &String,
    amount: Vec<Uint128>,
    signature: &[u8],
    public_key: &[u8],
) -> StdResult<bool> {
    let amount_string = amount
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let msg = format!("{},{},{}", account.to_string(), amount_string, asset_string);
    let msg_buf = msg.as_bytes();
    let keccak_digest = Keccak256::digest(msg_buf);
    let hash = keccak_digest.as_slice();
    let result = deps.api.secp256k1_verify(hash, signature, public_key);
    match result {
        Ok(true) => Ok(true),
        _ => Ok(false),
    }
}

pub fn is_valid_lootbox_signature(
    deps: &DepsMut,
    account: &Addr,
    asset_string: &String,
    amount: Vec<Uint128>,
    signature: &[u8],
    public_key: &[u8],
) -> StdResult<bool> {
    let amount_string = amount
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let msg = format!(
        "lootbox,{},{},{}",
        account.to_string(),
        amount_string,
        asset_string
    );
    let msg_buf = msg.as_bytes();
    let keccak_digest = Keccak256::digest(msg_buf);
    let hash = keccak_digest.as_slice();
    let result = deps.api.secp256k1_verify(hash, signature, public_key);
    match result {
        Ok(true) => Ok(true),
        _ => Ok(false),
    }
}
