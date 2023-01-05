use cosmwasm_std::Binary;

pub fn check_secp256k1_public_key(key: &Binary) -> bool {
    // Checking the submitted key adheres to the key standards.
    // The 65 byte key is not planned for usage, only the 33 byte key.
    // https://www.npmjs.com/package/secp256k1
    key.len() == 33 && (key[0] == 0x02 || key[0] == 0x03)
}
