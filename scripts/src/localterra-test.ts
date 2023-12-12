import {
  LCDClient,
  MsgStoreCode,
  MnemonicKey,
  MsgInstantiateContract,
  MsgExecuteContract,
  Wallet,
} from "@terra-money/feather.js";
import * as process from "process";
import * as fs from "fs";
import * as path from "path";
import * as secp256k1 from "secp256k1";
import keccak256 from "keccak256";

console.log("Example usage: yarn start src/test.ts --m1");

const IS_M1 = process.argv[2] === "--m1";
const M1_MODIFIER = IS_M1 ? "-aarch64" : "";

const SEED_PHRASE =
  "satisfy adjust timber high purchase tuition stool faith fine install that you unaware feed domain license impose boss human eager hat rent enjoy dawn";
const PROTOCOL_PHRASE =
  "quality vacuum heart guard buzz spike sight swarm shove special gym robust assume sudden deposit grid alcohol choice devote leader tilt noodle tide penalty";
const USER_PHRASE =
  "symbol force gallery make bulk round subway violin worry mixture penalty kingdom boring survey tool fringe patrol sausage hard admit remember broken alien absorb";
const PUBLICKEY = "Am1jWYX2c5sI7ukqcso6kdN9UIxo2amNTyxosGY/n/v6";
const PRIVATEKEY = "fQjH2LwnD650FSDPxja027umgzNlSBxDK5ReFhRZsJQ=";

const chainID = "localterra";
const prefix = "terra";

const FACTORY_CONTRACT = "genie-airdrop-factory";
const CONTRACT = "genie-airdrop";
const CW20 = "cw20-base";

const asset_info_luna = {
  native_token: {
    denom: "uluna",
  },
};

if (!SEED_PHRASE) {
  console.log("Missing SEED_PHRASE env var");
  process.exit(1);
}
if (!PROTOCOL_PHRASE) {
  console.log("Missing PROTOCOL_PHRASE env var");
  process.exit(1);
}
if (!USER_PHRASE) {
  console.log("Missing USER_PHRASE env var");
  process.exit(1);
}
if (!PRIVATEKEY) {
  console.log("Missing PRIVATEKEY env var");
  process.exit(1);
}
if (!PUBLICKEY) {
  console.log("Missing PUBLICKEY env var");
  process.exit(1);
}

const terra = new LCDClient({
  localterra: {
    // key must be the chainID
    lcd: "http://localhost:1317",
    chainID: "localterra",
    gasAdjustment: 1.75,
    gasPrices: { uluna: 0.15 },
    prefix: "terra",
  },
});

const key = new MnemonicKey({
  mnemonic: SEED_PHRASE,
});
const hallwallet = terra.wallet(key);
const key2 = new MnemonicKey({
  mnemonic: PROTOCOL_PHRASE,
});
const protocolWallet = terra.wallet(key2);
const key3 = new MnemonicKey({
  mnemonic: USER_PHRASE,
});
const userWallet = terra.wallet(key3);

const factoryFile = fs.readFileSync(
  path.resolve(
    __dirname,
    "..",
    "..",
    "artifacts",
    FACTORY_CONTRACT.replace(/-/g, "_") + M1_MODIFIER + ".wasm"
  )
);
const file = fs.readFileSync(
  path.resolve(
    __dirname,
    "..",
    "..",
    "artifacts",
    CONTRACT.replace(/-/g, "_") + M1_MODIFIER + ".wasm"
  )
);
const cw20File = fs.readFileSync(
  path.resolve(
    __dirname,
    "..",
    "..",
    "artifacts",
    CW20.replace(/-/g, "_") + M1_MODIFIER + ".wasm"
  )
);

async function setupCW20(wallet: Wallet) {
  const upload = new MsgStoreCode(
    wallet.key.accAddress(prefix),
    Buffer.from(cw20File).toString("base64")
  );
  const tx = await wallet.createAndSignTx({
    msgs: [upload],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
  const code = parseInt(
    res.logs[0].events
      .find((x) => x.type === "store_code")
      ?.attributes.find((x) => x.key == "code_id")?.value ?? "0"
  );
  console.log("code", code);

  const initMsg = {
    name: "Astro",
    symbol: "ASTRO",
    decimals: 6,
    initial_balances: [
      {
        address: wallet.key.accAddress(prefix),
        amount: "1000000000000",
      },
    ],
    mint: {
      minter: wallet.key.accAddress(prefix),
      cap: "1000000000000",
    },
  };

  const instantiate = new MsgInstantiateContract(
    wallet.key.accAddress(prefix),
    undefined,
    code,
    initMsg,
    {},
    "astro"
  );

  const tx2 = await wallet.createAndSignTx({
    msgs: [instantiate],
    chainID: chainID,
  });
  console.log(tx2);
  console.log("----------------------------------");
  const res2 = await terra.tx.broadcast(tx2, chainID);
  console.log(res2);
  const cw20Contract =
    res2.logs[0].events
      .find((x) => x.type === "instantiate")
      ?.attributes.find((x) => x.key == "_contract_address")?.value ?? "";
  console.log("cw20Contract", cw20Contract);

  return cw20Contract;
}
async function uploadContract(wallet: Wallet) {
  const uploadFactory = new MsgStoreCode(
    wallet.key.accAddress(prefix),
    Buffer.from(factoryFile).toString("base64")
  );
  const upload = new MsgStoreCode(
    wallet.key.accAddress(prefix),
    Buffer.from(file).toString("base64")
  );

  const tx = await wallet.createAndSignTx({
    msgs: [uploadFactory, upload],
    chainID: chainID,
  });

  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
  const factoryCode = parseInt(
    res.logs[0].events
      .find((x) => x.type === "store_code")
      ?.attributes.find((x) => x.key == "code_id")?.value ?? "0"
  );
  const contractCode = parseInt(
    res.logs[1].events
      .find((x) => x.type === "store_code")
      ?.attributes.find((x) => x.key == "code_id")?.value ?? "0"
  );
  console.log("factoryCode", factoryCode);
  console.log("contractCode", contractCode);
  return [factoryCode, contractCode];
}
async function instantiateFactory(
  wallet: Wallet,
  factoryCode: number,
  contractCode: number
) {
  const initMsg = { airdrop_code_id: contractCode, public_key: PUBLICKEY };

  const instantiateFactory = new MsgInstantiateContract(
    wallet.key.accAddress(prefix),
    undefined,
    factoryCode,
    initMsg,
    {},
    "factory"
  );

  const tx = await wallet.createAndSignTx({
    msgs: [instantiateFactory],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
  const factoryContract =
    res.logs[0].events
      .find((x) => x.type === "instantiate")
      ?.attributes.find((x) => x.key == "_contract_address")?.value ?? "";
  console.log("factoryContract", factoryContract);
  return factoryContract;
}
async function updateAirdropConfig(
  wallet: Wallet,
  factoryContract: string,
  contractCode: number
) {
  const updateConfig = new MsgExecuteContract(
    wallet.key.accAddress(prefix),
    factoryContract,
    {
      update_airdrop_config: {
        config: {
          airdrop_type: "asset",
          code_id: contractCode,
          is_disabled: false,
        },
      },
    },
    {}
  );

  const tx = await wallet.createAndSignTx({
    msgs: [updateConfig],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
}

async function createAirdrop(
  wallet: Wallet,
  factoryContract: string,
  asset: object,
  allocated_amounts: number[],
  from_timestamp: number,
  to_timestamp: number,
  campaign_id: string
) {
  const createAirdrop = new MsgExecuteContract(
    wallet.key.accAddress(prefix),
    factoryContract,
    {
      create_airdrop: {
        asset_info: asset,
        from_timestamp: from_timestamp,
        to_timestamp: to_timestamp,
        allocated_amounts: allocated_amounts.map((x) => x.toString()),
        campaign_id: campaign_id,
      },
    },
    {}
  );
  const tx = await wallet.createAndSignTx({
    msgs: [createAirdrop],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
  const airdropContract =
    res.logs[0].events
      .find((x) => x.type === "instantiate")
      ?.attributes.find((x) => x.key == "_contract_address")?.value ?? "";
  console.log("airdropContract", airdropContract);
  return airdropContract;
}
async function increaseIncentives(
  wallet: Wallet,
  token_contract: string,
  amount: number,
  airdropContract: string
) {
  const astroSend = {
    send: {
      contract: airdropContract,
      amount: amount.toString(),
      msg: Buffer.from(
        JSON.stringify({
          increase_incentives: {},
        })
      ).toString("base64"),
    },
  };
  const sendTokens = new MsgExecuteContract(
    wallet.key.accAddress(prefix),
    token_contract,
    astroSend,
    {}
  );

  const tx = await wallet.createAndSignTx({
    msgs: [sendTokens],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
}

async function topupIncentives(
  wallet: Wallet,
  token_contract: string,
  amount: number[],
  airdropContract: string
) {
  const astroSend = {
    send: {
      contract: airdropContract,
      amount: amount.reduce((a, b) => a + b, 0).toString(),
      msg: Buffer.from(
        JSON.stringify({
          increase_incentives: {
            topup_amounts: amount.map((x) => x.toString()),
          },
        })
      ).toString("base64"),
    },
  };
  const sendTokens = new MsgExecuteContract(
    wallet.key.accAddress(prefix),
    token_contract,
    astroSend,
    {}
  );

  const tx = await wallet.createAndSignTx({
    msgs: [sendTokens],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
}

async function increaseLunaIncentives(
  wallet: Wallet,
  airdropContract: string,
  amount: number
) {
  const increaseIncentives = new MsgExecuteContract(
    wallet.key.accAddress(prefix),
    airdropContract,
    {
      increase_incentives: {},
    },
    { uluna: amount.toString() }
  );
  const tx = await wallet.createAndSignTx({
    msgs: [increaseIncentives],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
}
async function claim(
  wallet: Wallet,
  airdropContract: string,
  amounts: number[]
) {
  const private_key = Buffer.from(PRIVATEKEY ?? "", "base64");
  const account = wallet.key.accAddress(prefix);
  const claimsContract = airdropContract;
  const amountstr = amounts
    .map((x) => x.toLocaleString("fullwide", { useGrouping: false }))
    .join(",");
  const claimstr = account + "," + amountstr + "," + claimsContract;
  const msg = keccak256(Buffer.from(claimstr));
  const sigObj = secp256k1.ecdsaSign(msg, private_key);
  const signature = Buffer.from(sigObj.signature).toString("base64");

  let amounts_string_array = amounts.map((amt) =>
    amt.toLocaleString("fullwide", { useGrouping: false })
  );

  let lootbox_info = amounts
    .map((amt) => Math.ceil(amt / 10))
    .map((amt) => amt.toLocaleString("fullwide", { useGrouping: false }));

  let claim_payload = JSON.stringify({
    claim_amounts: amounts_string_array,
    signature: signature,
    lootbox_info: lootbox_info,
  });
  claim_payload = Buffer.from(claim_payload).toString("base64");

  const claim = new MsgExecuteContract(
    wallet.key.accAddress("terra"),
    airdropContract,
    {
      claim: {
        payload: claim_payload,
      },
    },
    {}
  );
  const tx = await wallet.createAndSignTx({
    msgs: [claim],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
  return res;
}

async function transferUnclaimedTokens(
  wallet: Wallet,
  airdropContract: string
) {
  const transferUnclaimed = new MsgExecuteContract(
    wallet.key.accAddress("terra"),
    airdropContract,
    {
      transfer_unclaimed_tokens: {
        recipient: wallet.key.accAddress("terra"),
      },
    },
    {}
  );
  const tx = await wallet.createAndSignTx({
    msgs: [transferUnclaimed],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
  return res;
}

async function mint() {
  const cw20 =
    "terra1j08452mqwadp8xu25kn9rleyl2gufgfjnv0sn8dvynynakkjukcqsc244x";

  const mintMsg = new MsgExecuteContract(
    hallwallet.key.accAddress(prefix),
    cw20,
    {
      mint: {
        recipient: protocolWallet.key.accAddress(prefix),
        amount: "200000000000",
      },
    },
    {}
  );

  const tx = await hallwallet.createAndSignTx({
    msgs: [mintMsg],
    chainID: chainID,
  });

  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
}

async function wait(ms: number) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

async function waitUntil(s: number) {
  const timeUntil = s * 1000;
  console.log("timeUntil", timeUntil);
  const waiting_time = timeUntil - Date.now() + 5000;
  // try to wait until RPC/LCD and feather.js is updated with a time in the future
  // takes at least 1 minute to update properly
  console.log("waiting_time", waiting_time);
  return new Promise((resolve) => {
    setTimeout(resolve, waiting_time);
  });
}

const expectError = (message) => (err: Error) => {
  if (err.message !== "Error not thrown") {
    console.log(message);
  } else {
    throw new Error("Error not thrown");
  }
};
const throwErr = (_: any) => {
  throw new Error("Error not thrown");
};

testall();

async function testall() {
  console.log("SETUP CW20 TOKEN");
  const TOKEN_CONTRACT = await setupCW20(protocolWallet);

  console.log("UPLOADING CONTRACTS");
  const factoryContract = await uploadContract(hallwallet).then(async (res) => {
    console.log("INSTANTIATING FACTORY");
    await wait(3000);
    const factoryContract = await instantiateFactory(
      hallwallet,
      res[0],
      res[1]
    );

    await wait(3000);
    await updateAirdropConfig(hallwallet, factoryContract, res[1]);
    return factoryContract;
  });

  // update airdrop config to use cw20 token

  console.log("TESTING MULTI TEST 1");
  await test1(factoryContract, TOKEN_CONTRACT).catch((err) => {
    console.log(err);
  });

  console.log("TESTING SINGLE TEST 1");
  await single_test1(factoryContract, TOKEN_CONTRACT).catch((err) => {
    console.log(err);
  });

  console.log("TESTING SINGLE TEST 2");
  await single_test2(factoryContract, TOKEN_CONTRACT).catch((err) => {
    console.log(err);
  });

  console.log("TESTING SINGLE TEST 3");
  await single_test3(factoryContract, TOKEN_CONTRACT).catch((err) => {
    console.log(err);
  });

  console.log("TESTING TOPUP TEST");
  await topup_test(factoryContract, TOKEN_CONTRACT).catch((err) => {
    console.log(err);
  });

  console.log("DONE TESTING");
}

/*
Test scenario for multi mission contract
- [ ]  User1 claims [2,0,0] astro
- [ ]  User1 claims [3,0,0] astro (Receives 1 astro)
- [ ]  User1 claims [1,0,0] astro (fail)
- [ ]  User1 claims [1,0,3] astro (fail)
- [ ]  User1 claims [3,0,0] astro (no fail)
- [ ]  User2 claims [5,5,5] astro
- [ ]  User3 claims [10,1,1] astro (Receives 2 + 1 + 1 astro)
- [ ]  User1 claims [4,0,0] astro (nofail, nothing left to claim)
- [ ]  User1 claims [4,0,1] astro (Receives 1 astro from mission 3)
50 - 10 - 6 - 7 = 27
*/
async function test1(factoryContract: string, TOKEN_CONTRACT: string) {
  const asset_info = {
    token: {
      contract_addr: TOKEN_CONTRACT,
    },
  };
  const starttime = Math.trunc(Date.now() / 1000 + 10);
  const endtime = Math.trunc(Date.now() / 1000 + 50);
  await wait(1500);

  const airdropContract = await createAirdrop(
    protocolWallet,
    factoryContract,
    asset_info,
    [10_000_000, 20_000_000, 20_000_000],
    starttime,
    endtime,
    "1"
  );
  await wait(1500);
  await increaseIncentives(
    protocolWallet,
    TOKEN_CONTRACT,
    50_000_000,
    airdropContract
  );
  await waitUntil(starttime);
  await claim(userWallet, airdropContract, [2_000_000, 0, 0]);
  await wait(1500); // Wait a bit for wallet nonce to update.
  await claim(userWallet, airdropContract, [3_000_000, 0, 0]);
  await wait(1500);
  await claim(userWallet, airdropContract, [1_000_000, 0, 0])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim"));
  await wait(1500);
  await claim(userWallet, airdropContract, [1_000_000, 0, 3_000_000])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim"));
  await wait(1500);
  // This should not error anymore
  await claim(userWallet, airdropContract, [3_000_000, 0, 0]);
  await wait(1500);
  await claim(
    protocolWallet,
    airdropContract,
    [5_000_000, 5_000_000, 5_000_000]
  );
  await wait(1500);
  await claim(hallwallet, airdropContract, [10_000_000, 1_000_000, 1_000_000]);
  await wait(1500);
  await claim(userWallet, airdropContract, [4_000_000, 0, 0]);
  await wait(1500);
  await claim(userWallet, airdropContract, [4_000_000, 0, 1_000_000]);

  await waitUntil(endtime);
  await transferUnclaimedTokens(protocolWallet, airdropContract);
  return airdropContract;
}

async function single_test1(factoryContract: string, TOKEN_CONTRACT: string) {
  const asset_info = {
    token: {
      contract_addr: TOKEN_CONTRACT,
    },
  };
  const starttime = Math.trunc(Date.now() / 1000 + 10);
  const endtime = Math.trunc(Date.now() / 1000 + 30);

  await wait(1500);

  const airdropContract = await createAirdrop(
    protocolWallet,
    factoryContract,
    asset_info,
    [5000000],
    starttime,
    endtime,
    "2"
  );
  await wait(1500);
  await increaseIncentives(
    protocolWallet,
    TOKEN_CONTRACT,
    2000000,
    airdropContract
  );
  await wait(1500);
  await increaseIncentives(
    protocolWallet,
    TOKEN_CONTRACT,
    7000000,
    airdropContract
  );
  await waitUntil(starttime);
  await claim(userWallet, airdropContract, [2000000]);
  await wait(1500); // Wait a bit for wallet nonce to update.
  await claim(userWallet, airdropContract, [2000001]);
  await wait(1500);
  await claim(protocolWallet, airdropContract, [4000000]);
  await wait(1500);
  await claim(hallwallet, airdropContract, [1000000]);

  await waitUntil(endtime);
  await transferUnclaimedTokens(protocolWallet, airdropContract);
  return airdropContract;
}

async function single_test2(factoryContract: string, TOKEN_CONTRACT: string) {
  const starttime = Math.trunc(Date.now() / 1000 + 10);
  const endtime = Math.trunc(Date.now() / 1000 + 30);
  const airdropContract = await createAirdrop(
    protocolWallet,
    factoryContract,
    asset_info_luna,
    [5000000],
    starttime,
    endtime,
    "3"
  );
  await wait(1500);
  await increaseLunaIncentives(protocolWallet, airdropContract, 2000000);
  await wait(1500);
  await claim(userWallet, airdropContract, [2000000])
    .then(throwErr)
    .catch(expectError("Error is thrown for claim before start"));
  await waitUntil(starttime);
  await claim(userWallet, airdropContract, [2000000])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim due to no more tokens")); // prettier-ignore

  await waitUntil(endtime);
  transferUnclaimedTokens(protocolWallet, airdropContract);
}

async function single_test3(factoryContract: string, TOKEN_CONTRACT: string) {
  const asset_info = {
    token: {
      contract_addr: TOKEN_CONTRACT,
    },
  };
  await wait(5000);
  const starttime = Math.trunc(Date.now() / 1000 + 10);
  const endtime = Math.trunc(Date.now() / 1000 + 30);
  const airdropContract = await createAirdrop(
    protocolWallet,
    factoryContract,
    asset_info,
    [5000000],
    starttime,
    endtime,
    "4"
  );
  await wait(1500);
  await increaseIncentives(
    protocolWallet,
    TOKEN_CONTRACT,
    6000000,
    airdropContract
  );
  await waitUntil(starttime);
  await claim(userWallet, airdropContract, [2000000]);
  await waitUntil(endtime);
  await transferUnclaimedTokens(protocolWallet, airdropContract);
}

async function topup_test(factoryContract: string, TOKEN_CONTRACT: string) {
  const asset_info = {
    token: {
      contract_addr: TOKEN_CONTRACT,
    },
  };
  const starttime = Math.trunc(Date.now() / 1000 + 10);
  const endtime = Math.trunc(Date.now() / 1000 + 30);

  const airdropContract = await createAirdrop(
    protocolWallet,
    factoryContract,
    asset_info,
    [2000, 5000, 3000],
    starttime,
    endtime,
    "5"
  );

  await wait(1500);
  await increaseIncentives(
    protocolWallet,
    TOKEN_CONTRACT,
    10000,
    airdropContract
  );
  await waitUntil(starttime);
  await claim(userWallet, airdropContract, [3000, 3000, 3000]);
  await wait(1500);

  // last claimer should activate and claim additional 1000
  await topupIncentives(
    protocolWallet,
    TOKEN_CONTRACT,
    [2000, 2000, 2000],
    airdropContract
  );
  await wait(1500);

  await claim(userWallet, airdropContract, [4000, 20000, 20000]);
  await waitUntil(endtime);
  await transferUnclaimedTokens(protocolWallet, airdropContract);

  return airdropContract;
}
