# `genie-airdrop`

## Instantiating

There is a one-to-one relationship between campaigns and `genie-airdrop` contracts. Every new campaign should instantiate a new `genie-airdrop` contract by executing the `CreateAirdrop` message to the `genie-airdrop-factory` contract:

```js
// Example json to send:
{
  "create_airdrop": {
    "asset_info": {
      "token": {
        "contract_addr": "..."
      }
    },
    "from_timestamp": 1693800000,
    "to_timestamp": 1695268800,
    "allocated_amounts": [
      "1000000000000",
      "4000000000000"
    ],
    "campaign_id": "..."
  }
}
```

- `asset_info`: the asset that claimers will claim, which can either be a native coin or CW20 token
- `from_timestamp`: the unix timestamp for the start of this campaign
- `to_timestamp`: the unix timestamp for the end of this campaign
- `allocated_amounts`: the campaign budget (amount of assets) given to each mission; the sum of this array is the total budget of the overall campaign
- `campaign_id`: metadata linking this contract to the campaign data stored off-chain

## Migrating

The `genie-airdrop-factory` factory contract stores the code ID of the `genie-airdrop` binary, and during the execution of `CreateAirdrop`, it will use this stored code ID to instantiate a new `genie-airdrop` contract.

If there is a new code ID for `genie-airdrop` (ie. when it is migrated), we will need to update `genie-airdrop-factory` to point to the new code ID by executing the `UpdateConfig` message:

```js
// Example json to send:
{
  "update_config": {
    "airdrop_code_id": 420
  }
}
```

## Flow Chart

![Flow chart](../../docs/imgs/genie-claims-flow.png)

1. User requests server to claim
2. The server, which we regard as being the main source of truth, verifies and issues a "signed certificate" which contains the following information:
   1. Claimer's address
   2. Identifier of the assets to claim
   3. Amount of assets to claim
3. The user executes the claim by passing this "signed certificate" to the smart contract
4. The smart contract verifies the "signed certificate" and transfers the assets claimed to the user if valid

## Phases

![State Machine](../../docs/imgs/state%20machine%20genie.png)

To prevent abuse by both *campaign developers* (those who instantiate this contract) and *claimers*, there are four different phases, with each having certain restrictions on what actions can be made:

1. **Not started** - the campaign has not yet started (ie. before `from_timestamp`):
   - Campaign developers can deposit rewards
   - Campaign developers can withdraw rewards
   - Claimers cannot claim rewards
2. **Invalid / Failed** - the campaign is currently ongoing (ie. within `from_timestamp` and `to_timestamp`) but the campaign developers did not deposit enough rewards to meet the `allocated_amounts`:
   - Campaign developers can withdraw rewards
   - Claimers cannot claim rewards
3. **Ongoing** - the campaign is currently ongoing (ie. within `from_timestamp` and `to_timestamp`) and the campaign developers has deposited enough rewards to meet the `allocated_amounts`:
   - Campaign developers cannot withdraw rewards
   - Claimers can claim rewards
4. **Ended** - the campaign is currently ended (ie. after `to_timestamp`):
   - Campaign developers can withdraw rewards
   - Claimers cannot claim rewards
