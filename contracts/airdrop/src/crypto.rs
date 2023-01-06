use cosmwasm_std::{Addr, DepsMut, StdResult, Uint128};
use sha3::{Digest, Keccak256};

pub fn is_valid_signature(
    deps: &DepsMut,
    account: &Addr,
    asset_string: &String,
    amount: Uint128,
    signature: &[u8],
    public_key: &[u8],
) -> StdResult<bool> {
    let msg = format!("{},{},{}", account.to_string(), amount, asset_string,);
    let msg_buf = msg.as_bytes();
    let keccak_digest = Keccak256::digest(msg_buf);
    let hash = keccak_digest.as_slice();
    let result = deps.api.secp256k1_verify(&hash, &signature, &public_key);
    match result {
        Ok(true) => Ok(true),
        _ => Ok(false),
    }
}
