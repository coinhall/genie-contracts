# Setting up tests

Add to .env file in config/ . Make sure these seed phrases are not linked to wallets on the mainnet containing money.

```
SEED_PHRASE = add twenty four word seed phrase
PROTOCOL_PHRASE = add twenty four word seed phrase
USER_PHRASE = add twenty four word seed phrase
PUBLICKEY = anyBase64+secp256k1PublicKey+in/Base64
PRIVATEKEY = matching+secp256k1PublicKey+in/Base64
```

Go to https://faucet.terra.money and make sure all these 3 wallets have some amount of testnet luna in them. Also, head to https://app.astroport.fi/swap, change to testnet wallet, and swap for some astro tokens on the testnet (>100 astro is enough) on the protocol wallet.

For publickey, privatekey pair, DO NOT use the key pair below on mainnet.

```
PUBLICKEY = A59iiunFlPQJGnIWvgJlUIcADoSDHZ4ROcZIYhldJfvD
PRIVATEKEY = /QH1Vgg0kk/S0xip2zLyW0uaHFfcYln6N6MmOnoIJBI=
```

# Run

To run:

```
yarn start src/test.ts
```
