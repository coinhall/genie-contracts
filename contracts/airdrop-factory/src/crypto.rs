use cosmwasm_std::Binary;

pub fn check_secp256k1_public_key(key: &Binary) -> bool {
    (key.len() == 33 && (key[0] == 0x02 || key[0] == 0x03)) || (key.len() == 65 && key[0] == 0x04)
}
