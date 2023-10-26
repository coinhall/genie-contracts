# `genie-contracts`

**🧞 Powering [Genie campaigns](https://genie.coinhall.org).**

## Contracts

This repository has two main contracts:

1. [**`genie-airdrop`**](./contracts/genie-airdrop/README.md): the contract to be instantiated per campaign, and which end users claim rewards from
   - `phoenix-1`: `1942` (code ID)
   - `osmosis-1`: `256` (code ID)
2. [**`genie-airdrop-factory`**](./contracts/genie-airdrop-factory/README.md): the factory contract to instantiate the `genie-airdrop` contract
   - `phoenix-1`: `1683` (code ID), `terra1dcfnzx0lh27w7rjh69e6kl5838dc7yyjj3r3h84jmkjjf08kknuqxr0x3r` (contract)
   - `osmosis-1`: `194` (code ID), `osmo1vjq3e3fp4fcdnd83nmxk5764mlewwxjejsjmg9jyehgh5zklj0pslvs3na` (contract)

Refer to their respective READMEs for more information about how they work.

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

Run in the root of this project to produce an optimised build in the `artifacts` directory:

```sh
./build.sh
```

## Testing

See [`./scripts`](./scripts/README.md) for details of the full E2E test using the `pisco-1` testnet.
