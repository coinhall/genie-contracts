# `genie-contracts`

**Powering [Genie campaigns](https://genie.coinhall.org).**

## Contracts

- [`genie-airdrop-factory`](contracts/genie-airdrop-factory/README.md): factory contract for `genie-airdrop`
  - Mainnet code ID: `970`
  - Mainnet contract: `terra1dcfnzx0lh27w7rjh69e6kl5838dc7yyjj3r3h84jmkjjf08kknuqxr0x3r`
- [`genie-airdrop`](contracts/genie-airdrop/README.md): contract which users claim rewards from
  - Mainnet code ID: `969`

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
