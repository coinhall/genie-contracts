# Scripts

This file contains useful scripts for testing out the Genie contracts.

## Generating Keys

```bash
# WARNING: the private key must be kept as a sensitive secret in production.
yarn start src/keygen.ts
```

## Signing Messages

```bash
yarn start src/sign.ts <PUBLIC_KEY> <PRIVATE_KEY>
```

## Testing Using Testnet

Follow the below steps to test the contracts using testnet.

### Setup

1. Create a `.env` file in `/config` dir:

   ```sh
   # WARNING: make sure these secrets are not production secrets!
   SEED_PHRASE='add twenty four word seed phrase'
   PROTOCOL_PHRASE='add twenty four word seed phrase'
   USER_PHRASE='add twenty four word seed phrase'
   PUBLICKEY='anyBase64+secp256k1PublicKey+in/Base64'
   PRIVATEKEY='matching+secp256k1PublicKey+in/Base64'
   ```

2. Create three new testnet wallets through terrastation
3. Head to <https://faucet.terra.money> and claim some testnet Luna
4. Head to <https://app.astroport.fi/swap>, change to `testnet`, and swap Luna for some Astro tokens

### Running

1. Ensure the wasm files are built; if not, run `./build.sh` in the root of the project
2. Then, run `yarn start src/test.ts`

> The testnet RPC may occasionally take a longer time to process incoming transactions. A delay of 6s is added between each transaction, and a minimum delay of 60s is added whenever it is needed to wait until a certain timestamp. This covers for most cases, but the tests might still fail for this reason. For more accurate testing, and to avoid spamming the testnet, a localnet could be setup in the future.
