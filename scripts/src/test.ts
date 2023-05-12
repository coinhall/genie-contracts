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

const SEED_PHRASE = process.env.SEED_PHRASE;
const PROTOCOL_PHRASE = process.env.PROTOCOL_PHRASE;
const USER_PHRASE = process.env.USER_PHRASE;
const PUBLICKEY = process.env.PUBLICKEY;
const PRIVATEKEY = process.env.PRIVATEKEY;

const FACTORY_CONTRACT = "genie-airdrop-factory";
const CONTRACT = "genie-airdrop";
const TOKEN_CONTRACT =
  "terra167dsqkh2alurx997wmycw9ydkyu54gyswe3ygmrs4lwume3vmwks8ruqnv";
const asset_info = {
  token: {
    contract_addr: TOKEN_CONTRACT,
  },
};
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
  "pisco-1": {
    lcd: "https://pisco-lcd.terra.dev",
    chainID: "pisco-1",
    gasAdjustment: 1.75,
    gasPrices: { uluna: 0.015 },
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
async function uploadContract(wallet: Wallet) {
  const uploadFactory = new MsgStoreCode(
    wallet.key.accAddress("terra"),
    Buffer.from(factoryFile).toString("base64")
  );
  const upload = new MsgStoreCode(
    wallet.key.accAddress("terra"),
    Buffer.from(file).toString("base64")
  );

  const tx = await wallet.createAndSignTx({
    msgs: [uploadFactory, upload],
    chainID: "pisco-1",
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, "pisco-1");
  console.log(res);
  const factoryCode = parseInt(res.logs[0].events[1].attributes[1].value);
  const contractCode = parseInt(res.logs[1].events[1].attributes[1].value);
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
    wallet.key.accAddress("terra"),
    undefined,
    factoryCode,
    initMsg,
    {},
    "factory"
  );
  const tx = await wallet.createAndSignTx({
    msgs: [instantiateFactory],
    chainID: "pisco-1",
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, "pisco-1");
  console.log(res);
  const factoryContract = res.logs[0].events[0].attributes[0].value;
  console.log("factoryContract", factoryContract);
  return factoryContract;
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
    wallet.key.accAddress("terra"),
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
    chainID: "pisco-1",
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, "pisco-1");
  console.log(res);
  const airdropContract = res.logs[0].events[1].attributes[0].value;
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
    wallet.key.accAddress("terra"),
    token_contract,
    astroSend,
    {}
  );

  const tx = await wallet.createAndSignTx({
    msgs: [sendTokens],
    chainID: "pisco-1",
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, "pisco-1");
  console.log(res);
}
async function increaseLunaIncentives(
  wallet: Wallet,
  airdropContract: string,
  amount: number
) {
  const increaseIncentives = new MsgExecuteContract(
    wallet.key.accAddress("terra"),
    airdropContract,
    {
      increase_incentives: {},
    },
    { uluna: amount.toString() }
  );
  const tx = await wallet.createAndSignTx({
    msgs: [increaseIncentives],
    chainID: "pisco-1",
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, "pisco-1");
  console.log(res);
}
async function claim(
  wallet: Wallet,
  airdropContract: string,
  amount: number[]
) {
  const private_key = Buffer.from(PRIVATEKEY ?? "", "base64");
  const account = wallet.key.accAddress("terra");
  const claimsContract = airdropContract;
  const amountstr = amount
    .map((x) => x.toLocaleString("fullwide", { useGrouping: false }))
    .join(",");
  const claimstr = account + "," + amountstr + "," + claimsContract;
  const msg = keccak256(Buffer.from(claimstr));
  const sigObj = secp256k1.ecdsaSign(msg, private_key);
  const signature = Buffer.from(sigObj.signature).toString("base64");
  const claim = new MsgExecuteContract(
    wallet.key.accAddress("terra"),
    airdropContract,
    {
      claim: {
        signature: signature,
        claim_amounts: amount.map((x) => x.toString()),
      },
    },
    {}
  );
  const tx = await wallet.createAndSignTx({
    msgs: [claim],
    chainID: "pisco-1",
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, "pisco-1");
  console.log(res);
  return res;
}

async function transferUnclaimedTokens(
  wallet: Wallet,
  airdropContract: string,
  amount: number
) {
  const transferUnclaimed = new MsgExecuteContract(
    wallet.key.accAddress("terra"),
    airdropContract,
    {
      transfer_unclaimed_tokens: {
        amount: amount.toString(),
        recipient: wallet.key.accAddress("terra"),
      },
    },
    {}
  );
  const tx = await wallet.createAndSignTx({
    msgs: [transferUnclaimed],
    chainID: "pisco-1",
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, "pisco-1");
  console.log(res);
  return res;
}

async function wait(ms: number) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

async function waitUntil(s: number) {
  const timeUntil = s * 1000;
  console.log("timeUntil", timeUntil);
  const waiting_time =
    timeUntil - Date.now() > 0 ? timeUntil - Date.now() + 60000 : 60000;
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
  console.log("UPLOADING CONTRACTS");
  const factoryContract = await uploadContract(hallwallet).then(async (res) => {
    await wait(6000);
    console.log("INSTANTIATING FACTORY");
    return instantiateFactory(hallwallet, res[0], res[1]);
  });
  await wait(6000);

  // const factoryContract =
  //   "terra1ydwlh3auwwhn7xl4fn5zaeqx7xktmd9kqp0la4da3zxd7t6frjws2j50st";

  console.log("TESTING MULTI TEST 1");
  await test1(factoryContract).catch((err) => {
    console.log(err);
  });
  await wait(6000);

  console.log("TESTING SINGLE TEST 1");
  await single_test1(factoryContract).catch((err) => {
    console.log(err);
  });
  await wait(6000);

  console.log("TESTING SINGLE TEST 2");
  await single_test2(factoryContract).catch((err) => {
    console.log(err);
  });
  await wait(6000);

  console.log("TESTING SINGLE TEST 3");
  await single_test3(factoryContract).catch((err) => {
    console.log(err);
  });
  await wait(6000);

  console.log("DONE TESTING");
}

/*
Test scenario for multi mission contract
- [ ]  User1 claims [2,0,0] astro
- [ ]  User1 claims [3,0,0] astro (Receives 1 astro)
- [ ]  User1 claims [1,0,0] astro (fail)
- [ ]  User1 claims [1,0,3] astro (fail)
- [ ]  User1 claims [3,0,0] astro (fail)
- [ ]  User2 claims [5,5,5] astro
- [ ]  User3 claims [10,1,1] astro (Receives 2 + 1 + 1 astro)
- [ ]  User1 claims [4,0,0] astro (fail, nothing left to claim)
- [ ]  User1 claims [4,0,1] astro (Receives 1 astro from mission 3)
50 - 10 - 6 - 7 = 27
*/
async function test1(factoryContract: string) {
  const starttime = Math.trunc(Date.now() / 1000 + 60);
  const endtime = Math.trunc(Date.now() / 1000 + 300);
  await wait(6000);

  const airdropContract = await createAirdrop(
    protocolWallet,
    factoryContract,
    asset_info,
    [10_000_000, 20_000_000, 20_000_000],
    starttime,
    endtime,
    "1"
  );
  await wait(6000);
  await increaseIncentives(
    protocolWallet,
    TOKEN_CONTRACT,
    50_000_000,
    airdropContract
  );
  await waitUntil(starttime);
  await claim(userWallet, airdropContract, [2_000_000, 0, 0]);
  await wait(6000); // Wait a bit for wallet nonce to update.
  await claim(userWallet, airdropContract, [3_000_000, 0, 0]);
  await wait(6000);
  await claim(userWallet, airdropContract, [1_000_000, 0, 0])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim"));
  await wait(6000);
  await claim(userWallet, airdropContract, [1_000_000, 0, 3_000_000])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim"));
  await wait(6000);
  await claim(userWallet, airdropContract, [3_000_000, 0, 0])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim"));
  await wait(6000);
  await claim(
    protocolWallet,
    airdropContract,
    [5_000_000, 5_000_000, 5_000_000]
  );
  await wait(6000);
  await claim(hallwallet, airdropContract, [10_000_000, 1_000_000, 1_000_000]);
  await wait(6000);
  await claim(userWallet, airdropContract, [4_000_000, 0, 0])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim"));
  await wait(6000);
  await claim(userWallet, airdropContract, [4_000_000, 0, 1_000_000]);
  await waitUntil(endtime);
  await transferUnclaimedTokens(protocolWallet, airdropContract, 27_000_000);
  return airdropContract;
}

async function single_test1(factoryContract: string) {
  const starttime = Math.trunc(Date.now() / 1000 + 60);
  const endtime = Math.trunc(Date.now() / 1000 + 500);

  await wait(6000);

  const airdropContract = await createAirdrop(
    protocolWallet,
    factoryContract,
    asset_info,
    [5000000],
    starttime,
    endtime,
    "1"
  );
  await wait(6000);
  await increaseIncentives(
    protocolWallet,
    TOKEN_CONTRACT,
    2000000,
    airdropContract
  );
  await wait(6000);
  await increaseIncentives(
    protocolWallet,
    TOKEN_CONTRACT,
    7000000,
    airdropContract
  );
  await wait(6000);
  await transferUnclaimedTokens(protocolWallet, airdropContract, 4000000);
  await waitUntil(starttime);
  await claim(userWallet, airdropContract, [2000000]);
  await wait(6000); // Wait a bit for wallet nonce to update.
  await claim(userWallet, airdropContract, [2000001]);
  await wait(6000);
  await claim(protocolWallet, airdropContract, [4000000]);
  await wait(6000);
  await claim(hallwallet, airdropContract, [1000000])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim"));
  await wait(6000);
  return airdropContract;
}

async function single_test2(factoryContract: string) {
  const starttime = Math.trunc(Date.now() / 1000 + 60);
  const endtime = Math.trunc(Date.now() / 1000 + 250);
  const airdropContract = await createAirdrop(
    protocolWallet,
    factoryContract,
    asset_info_luna,
    [5000000],
    starttime,
    endtime,
    "2"
  );
  await wait(6000);
  await increaseLunaIncentives(protocolWallet, airdropContract, 2000000);
  await wait(6000);
  await claim(userWallet, airdropContract, [2000000])
    .then(throwErr)
    .catch(expectError("Error is thrown for claim before start"));
  await waitUntil(starttime);
  await claim(userWallet, airdropContract, [2000000])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim due to no more tokens")); // prettier-ignore

  await waitUntil(endtime);

  transferUnclaimedTokens(protocolWallet, airdropContract, 2000000);
}

async function single_test3(factoryContract: string) {
  const starttime = Math.trunc(Date.now() / 1000 + 50);
  const endtime = Math.trunc(Date.now() / 1000 + 150);
  const airdropContract = await createAirdrop(
    protocolWallet,
    factoryContract,
    asset_info,
    [5000000],
    starttime,
    endtime,
    "3"
  );
  await wait(6000);
  await increaseIncentives(
    protocolWallet,
    TOKEN_CONTRACT,
    6000000,
    airdropContract
  );
  await waitUntil(starttime);
  await claim(userWallet, airdropContract, [2000000]);
  await waitUntil(endtime);
  await transferUnclaimedTokens(protocolWallet, airdropContract, 4000000);
}
