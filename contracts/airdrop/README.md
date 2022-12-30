### Coming soon on [https://genie.coinhall.org/](https://genie.coinhall.org/)

# [Readme Under Construction]

## Claims Process:

1. User visits genie to request for a claim
2. Genie issues and signs a certificate [off-chain, no gas] to eligible user for the claim amount and address.
   Certificate details:

   1. Smart contract addr (required to prevent replay on other smart contracts)
   2. Claim amount
   3. User’s Address (Required to prevent someone else from claiming using this certificate)
   4. `secp256k1` Signature [https://www.npmjs.com/package/secp256k1](https://www.npmjs.com/package/secp256k1)

3. [Once only] User claims from smart contract using signed certificate.
   - In case of insufficient tokens in contract, user will be able to claim the remaining amount in the smart contract.
4. Smart contract sends coins to user 😄
