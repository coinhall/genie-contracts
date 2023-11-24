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

console.log("Example usage: yarn start src/localterra-nft-test.ts --m1");

const IS_M1 = process.argv[2] === "--m1";
const M1_MODIFIER = IS_M1 ? "-aarch64" : "";

// const SEED_PHRASE = process.env.SEED_PHRASE;
// const PROTOCOL_PHRASE = process.env.PROTOCOL_PHRASE;
// const USER_PHRASE = process.env.USER_PHRASE;
// const PUBLICKEY = process.env.PUBLICKEY;
// const PRIVATEKEY = process.env.PRIVATEKEY;
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
const NFT_CONTRACT = "genie-nft";
const CW721 = "cw721-base";

const TOKEN_CONTRACT =
  "terra1hm4y6fzgxgu688jgf7ek66px6xkrtmn3gyk8fax3eawhp68c2d5q74k9fw";
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
  localterra: {
    // key must be the chainID
    lcd: "http://localhost:1317",
    chainID: chainID,
    gasAdjustment: 1.75,
    gasPrices: { uluna: 0.15 },
    prefix: prefix,
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
const contracts = [FACTORY_CONTRACT, CONTRACT, NFT_CONTRACT, CW721];

async function uploadContract(wallet: Wallet, contracts: String[]) {
  const contractFile = contracts.map((x) => {
    return fs.readFileSync(
      path.resolve(
        __dirname,
        "..",
        "..",
        "artifacts",
        x.replace(/-/g, "_") + M1_MODIFIER + ".wasm"
      )
    );
  });
  const uploadMessages = contractFile.map((file) => {
    return new MsgStoreCode(
      wallet.key.accAddress(prefix),
      Buffer.from(file).toString("base64")
    );
  });

  let codes: number[] = [];
  for (const x of uploadMessages) {
    const tx = await wallet.createAndSignTx({
      msgs: [x],
      chainID: chainID,
    });
    const res = await terra.tx.broadcast(tx, chainID);
    const codeId = res.logs[0].events
      .find((x) => x.type === "store_code")
      ?.attributes.find((x) => x.key == "code_id")?.value;

    if (!codeId) {
      throw new Error("Code id not found");
    }
    codes.push(parseInt(codeId));
  }
  return codes;
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
  const factoryContract = res.logs[0].events
    .find((x) => x.type === "instantiate")
    ?.attributes.find((x) => x.key == "_contract_address")?.value;

  if (!factoryContract) {
    throw new Error("Factory contract not found");
  }
  console.log("factoryContract", factoryContract);
  return factoryContract;
}
async function instantiateNft(wallet: Wallet, nftCode: number) {
  const initMsg = {
    name: "NFT",
    symbol: "NFT",
    minter: wallet.key.accAddress(prefix),
  };

  const instantiateNft = new MsgInstantiateContract(
    wallet.key.accAddress(prefix),
    wallet.key.accAddress(prefix),
    nftCode,
    initMsg,
    {},
    "factory"
  );

  const tx = await wallet
    .createAndSignTx({
      msgs: [instantiateNft],
      chainID: chainID,
    })
    .catch((err) => {
      console.log(err);
    });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
  const nftContract =
    res.logs[0].events
      .find((x) => x.type === "instantiate")
      ?.attributes.find((x) => x.key == "_contract_address")?.value ?? "";
  console.log("nftContract", nftContract);
  return nftContract;
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
      ?.attributes.find((x) => x.key == "_contract_address")
      ?.value.toString() ?? "";
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
    wallet.key.accAddress(prefix),
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


async function wait(ms: number) {
  if (chainID === "localterra") {
    return;
  }

  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

async function waitUntil(s: number) {
  // if localnet, do not wait
  if (chainID === "localterra") {
    return;
  }

  const timeUntil = s * 1000;
  console.log("timeUntil", timeUntil);
  const waiting_time = timeUntil - Date.now() + 5000;
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
  const codes = await uploadContract(hallwallet, contracts);
  console.log("Contract codes: ", codes);

  await wait(3000);
  console.log("INSTANTIATING NFT");
  await instantiateNft(hallwallet, codes[3]);
  console.log("INSTANTIATING FACTORY");
  const factoryContract = await instantiateFactory(
    hallwallet,
    codes[0],
    codes[1]
  );

  await testnft1(factoryContract);

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
async function testnft1(airdropContract: string) {
  const starttime = Math.trunc(Date.now() / 1000 + 10);
  const endtime = Math.trunc(Date.now() / 1000 + 50);

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
