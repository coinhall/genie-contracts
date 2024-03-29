# Scripts

This directory contains useful scripts for generating keys and running the full E2E tests of the Genie contracts against the `pisco-1` testnet.

## Generating Keys

> **WARNING**: in production, the private key MUST be kept as a sensitive secret.

To generate a private-public key pair that this contract requires:

```bash
yarn start src/keygen.ts
```

## Running E2E Tests

> **Note**: The testnet RPC may occasionally take a longer time to process incoming transactions. A delay of 6s is added between each transaction, and a minimum delay of 60s is added whenever it is needed to wait until a certain timestamp. This covers for most cases, but the tests might still fail for this reason. For more accurate testing, and to avoid spamming the testnet, a localnet could be setup in the future.

These tests are written to target the `pisco-1` testnet.

### Setup

1. Create a `.env` file in `/config` dir (make sure these are NOT production secrets):

   ```sh
   # WARNING: make sure these are NOT production secrets!
   SEED_PHRASE='add twenty four word seed phrase'
   PROTOCOL_PHRASE='add twenty four word seed phrase'
   USER_PHRASE='add twenty four word seed phrase'
   # See 'Generating Keys' to generate these keys:
   PUBLICKEY='anyBase64+secp256k1PublicKey+in/Base64'
   PRIVATEKEY='matching+secp256k1PublicKey+in/Base64'
   ```

2. Create three new testnet wallets through terrastation
3. Head to <https://faucet.terra.money> and claim some testnet Luna
4. Head to <https://app.astroport.fi/swap>, change to `testnet`, and swap Luna for some Astro tokens

### Running

1. Run `./build.sh` in the root of the project to build the wasm files
2. Then, run `yarn start src/test.ts` and observe the results
