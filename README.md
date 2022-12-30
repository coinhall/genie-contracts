# Airdrop

The Airdrop contract facilitates direct claiming of CW20 tokens airdropped to users who participate in genie campaigns

## Contracts

| Name                                                 | Description                            |
| ---------------------------------------------------- | -------------------------------------- |
| [`genie-airdrop_factory`](contracts/airdrop-factory) | Factory contract for creating airdrops |
| [`genie-airdrop`](contracts/airdrop)                 | Genie airdrop contract                 |

- terraswap_factory

  Mainnet: `terraAbc12312312312312312312312312312312312312312312311231231231`

  Testnet: `terrabbc12312312312312312312312312312312312312312312311231231231`

  Mainnet (CodeID): xxx

  Testnet (CodeID): xxxx

- terraswap_pair

  Mainnet (CodeID): xxx

  Testnet (CodeID): xxxx

# Running this contract

## Prerequisites

You will need Rust 1.44.1+ with wasm32-unknown-unknown target installed.

```
rustc --version
cargo --version
rustup target list --installed
# if wasm32 is not listed above, run this
rustup target add wasm32-unknown-unknown
```

## Compiling for local testing

Run this in the contract file.

```
cargo wasm
```

## Compiling for the blockchain

Or for a blockchain-ready (compressed) build, run the following from the repository root:

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.6
```

The optimized contracts and checksums are generated in the artifacts/ directory.

The checksum can be compared against the one on https://terrasco.pe/mainnet/codes/123 (replace 123 with the Mainnet code id) or any other Terra blockchain explorer.
