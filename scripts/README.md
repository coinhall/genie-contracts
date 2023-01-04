### Node 14++ required

This file contains useful scripts for testing out the genie airdrops.

# Generating a private key

For publickey, privatekey pair, DO NOT use the key pair below on mainnet.

```bash
yarn start src/keygen.ts
#  Generating private key...
#  PUBLICKEY = A59iiunFlPQJGnIWvgJlUIcADoSDHZ4ROcZIYhldJfvD
#  PRIVATEKEY = /QH1Vgg0kk/S0xip2zLyW0uaHFfcYln6N6MmOnoIJBI=

# !! DO NOT USE THESE 2 KEYS ON THE MAINNET
```

# Trying out signing messages using the keypair

```bash
yarn start src/sign.ts <PUBLIC KEY> <PRIVATEKEY>

# yarn start src/sign.ts A59iiunFlPQJGnIWvgJlUIcADoSDHZ4ROcZIYhldJfvD /QH1Vgg0kk/S0xip2zLyW0uaHFfcYln6N6MmOnoIJBI=
# claimMsg =  {
#     claim: {
#         signature: 'DP4xA4uJ1fClkkN0hPVaLtVLja2bsnhdIbf66XJAiAMTXHKcdEOJHd6LR6Duv9NzL3BffigXXTMkGvdHF3O7Nw==',
#         claim_amount: '123123123'
#     }
# }
# isVerified =  true
```

# Setting up tests

Add to .env file in config/ . Make sure these seed phrases are not linked to wallets on the mainnet containing money.

```
SEED_PHRASE = add twenty four word seed phrase
PROTOCOL_PHRASE = add twenty four word seed phrase
USER_PHRASE = add twenty four word seed phrase
PUBLICKEY = anyBase64+secp256k1PublicKey+in/Base64
PRIVATEKEY = matching+secp256k1PublicKey+in/Base64
```

Create 3 new testnet wallets through terrastation. Go to https://faucet.terra.money and make sure all these 3 wallets have some amount of testnet luna in them. Also, head to https://app.astroport.fi/swap, change to `testnet`, and swap for some astro tokens on the testnet (>100 astro is enough) on the protocol wallet.

# Run tests on testnet

To run:

```
# Make sure the wasm files are in the artifacts folder, run build.sh
yarn start src/test.ts
```

\*Notes, sometimes the testnet RPC will take a longer time to process incoming transactions. Due to this, it cannot handle a burst of transactions in a short amount of time and it will complain that the nonce is incorrect, not enough sufficient coins (could be due to not enough testnet tokens also), contract doesn't exist, etc... . A delay of 6 seconds is added between each transaction, and a minimum delay of 60 is added whenever it is needed to wait until a certain timestamp. Despite this, the tests might still fail for this reason. For more accurate testing, and to avoid spamming the testnet, a suggestion will be to start a localnet and to use that instead with a new CW20 token instead of the testnet.
