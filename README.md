# `genie-contracts`

**Powering [Genie campaigns](https://genie.coinhall.org).**

## Contracts

- [`genie-airdrop-factory`](contracts/genie-airdrop-factory/README.md): factory contract for `genie-airdrop`
  - Testnet code ID: TODO
  - Testnet contract: `TODO`
  - Mainnet code ID: TODO
  - Mainnet contract: `TODO`
- [`genie-airdrop`](contracts/genie-airdrop/README.md): contract which users claim rewards from
  - Mainnet code ID: TODO
  - Testnet code ID: TODO

## Scripts

- [`test`](scripts/src/test.ts): to upload, instantiate and try out the contracts on testnet
- [`keygen`](scripts/src/keygen.ts): to generate random public and private `secp256k1` keys

## Installing

1. Install [Rust](https://www.rust-lang.org/tools/install) 1.44.1+
2. Install [Docker](https://docs.docker.com/get-docker/) for compiling and ensuring that builds have similar checksums
3. Install `wasm32-unknown-unknown` for rust

```sh
# Check rust versions
rustc --version
cargo --version
rustup target list --installed
# If `wasm32-unknown-unknown` is not listed, install it:
rustup target add wasm32-unknown-unknown
```

## Building

Run in the root of this project to produce an optimised build in the `~/artifacts` directory:

```sh
./build.sh
```
