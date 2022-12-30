# Genie Factory

The factory contract can perform creation of airdrop contract.

## InstantiateMsg

Creating a factory contract.

```json
{
  "airdrop_code_id": 123,
  "public_key": "base64/+/public+key/+/SECP256K1/////////////"
}
```

## ExecuteMsg

### `update_config`

```json
{
  "update_config": {
    "owner": "terra...",
    "airdrop_code_id": 123,
    "public_key": "base64/+/public+key/+/SECP256K1/////////////"
  }
}
```

### `create_airdrop`

Creating an airdrop contract using the factory.

```json
{
  "create_airdrop": {
    "asset_info": "terra123abc...",
    "from_timestamp": "1670222198",
    "from_timestamp": "1770222198"
  }
}
```

## QueryMsg [Section under construction]

### `config` [Section under construction]

```json
{
  "config": {}
}
```

### UpdateConfig

The factory contract owner can change relevant code IDs for future airdrop contract creation.

```json
{
    "update_config":
    {
        "owner": Option<HumanAddr>,
        "airdrop_code_id": Option<u64>
    }
}
```

### Create Airdrop

When a user execute `Create_airdrop` operation, it creates `Airdrop` contract.

Asset_info: token smart contract

From_timestamp: Unix timestamp in seconds to start the airdrop

To_timestamp: Unix timestamp in seconds to end the airdrop

```json
{
  "create_airdrop": {
    "asset_info": "terra123abc...",
    "from_timestamp": "1670222198",
    "from_timestamp": "1770222198"
  }
}
```
