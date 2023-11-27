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
const GENIE_CONTRACT = "genie-airdrop";
const GENIE_NFT_CONTRACT = "genie-nft";
const NFT_CONTRACT = "cw721-base";
const contracts = [
  FACTORY_CONTRACT,
  GENIE_CONTRACT,
  GENIE_NFT_CONTRACT,
  NFT_CONTRACT,
];
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

  const tx = await wallet.createAndSignTx({
    msgs: [instantiateNft],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
  const nftContract = res.logs[0].events
    .find((x) => x.type === "instantiate")
    ?.attributes.find((x) => x.key == "_contract_address")?.value;
  if (!nftContract) {
    throw new Error("NFT contract not found");
  }
  console.log("nftContract", nftContract);
  return nftContract;
}

async function instatiateAirdropNft(
  wallet: Wallet,
  genieNftCode: number,
  from_timestamp: number,
  to_timestamp: number,
  allocated_amounts: number[],
  nft_contract: string
) {
  const initMsg = {
    owner: wallet.key.accAddress(prefix),
    asset: {
      contract_addr: nft_contract,
    },

    public_key: PUBLICKEY,
    from_timestamp: from_timestamp,
    to_timestamp: to_timestamp,
    allocated_amounts: allocated_amounts.map((x) => x.toString()),
  };

  const instantiateGenieNft = new MsgInstantiateContract(
    wallet.key.accAddress(prefix),
    wallet.key.accAddress(prefix),
    genieNftCode,
    initMsg,
    {},
    "factory"
  );

  const tx = await wallet
    .createAndSignTx({
      msgs: [instantiateGenieNft],
      chainID: chainID,
    })
    .catch((err) => {
      console.log(err);
      throw err;
    });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
  const genieNftContract = res.logs[0].events
    .find((x) => x.type === "instantiate")
    ?.attributes.find((x) => x.key == "_contract_address")?.value;
  if (!genieNftContract) {
    throw new Error("NFT contract not found");
  }
  console.log("genieNftContract", genieNftContract);
  return genieNftContract;
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
  const airdropContract = res.logs[0].events
    .find((x) => x.type === "instantiate")
    ?.attributes.find((x) => x.key == "_contract_address")
    ?.value.toString();
  if (!airdropContract) {
    throw new Error("Airdrop contract not found");
  }
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

type MintInfo = {
  token_id: string;
  owner: string;
  token_uri: string;
};
let offset = 0;
function generateMintInfo(amount: number, receipient: string) {
  let mint_info: string[] = [];
  for (let i = offset; i < offset + amount; i++) {
    let mint_info_obj: MintInfo = {
      token_id: i.toString(),
      owner: receipient,
      token_uri:
        "https://ipfs.io/ipfs/QmeSjSinHpPnmXmspMjwiXyN6zS4E9zccariGR3jxcaWtq/" +
        i.toString(),
    };
    mint_info.push(
      Buffer.from(JSON.stringify(mint_info_obj)).toString("base64")
    );
  }
  offset += amount;
  return mint_info;
}

async function claim(
  wallet: Wallet,
  airdropContract: string,
  amounts: number[]
) {
  const account = wallet.key.accAddress(prefix);
  const amountToMint = amounts.reduce((a, b) => a + b, 0);
  const mint_info = generateMintInfo(amountToMint, account);

  const private_key = Buffer.from(PRIVATEKEY ?? "", "base64");
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
    mint_info: mint_info,
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
  const tx = await wallet
    .createAndSignTx({
      msgs: [claim],
      chainID: chainID,
    })
    .catch((err) => {
      console.log(err);
      throw err;
    });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID).catch((err) => {
    console.log(err);
    throw err;
  });
  console.log(res);
  return res;
}

async function transferNftContract(
  wallet: Wallet,
  nftContract: string,
  airdropContract: string
) {
  const transferNft = new MsgExecuteContract(
    wallet.key.accAddress(prefix),
    nftContract,
    {
      update_ownership: {
        transfer_ownership: {
          new_owner: nftContract,
        },
      },
    },
    {}
  );
  const tx = await wallet.createAndSignTx({
    msgs: [transferNft],
    chainID: chainID,
  });
  console.log(tx);
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, chainID);
  console.log(res);
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
  const nftContract = await instantiateNft(hallwallet, codes[3]);

  await testnft1(nftContract, factoryContract, codes);

  console.log("DONE TESTING");
}

async function testnft1(
  nft_contract: string,
  factoryContract: string,
  codes: number[]
) {
  const starttime = Math.trunc(Date.now() / 1000 + 10);
  const endtime = Math.trunc(Date.now() / 1000 + 50);
  let airdropContract = await instatiateAirdropNft(
    hallwallet,
    codes[2],
    starttime,
    endtime,
    [10, 100, 100],
    nft_contract
  );

  await transferNftContract(hallwallet, nft_contract, airdropContract);

  await waitUntil(starttime);
  await claim(userWallet, airdropContract, [2, 0, 0]);
  await wait(1500); // Wait a bit for wallet nonce to update.
  await claim(userWallet, airdropContract, [3, 0, 0]);
  await wait(1500);
  await claim(userWallet, airdropContract, [1, 0, 0])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim"));
  await wait(1500);
  await claim(userWallet, airdropContract, [1, 0, 3])
    .then(throwErr)
    .catch(expectError("Error is thrown for being unable to claim"));
  await wait(1500);
  // This should not error anymore
  await claim(userWallet, airdropContract, [3, 0, 0]);
  await wait(1500);
  await claim(
    protocolWallet,
    airdropContract,
    [5_000_000, 5_000_000, 5_000_000]
  );
  await wait(1500);
  await claim(hallwallet, airdropContract, [10, 1, 1]);
  await wait(1500);
  await claim(userWallet, airdropContract, [4, 0, 0]);
  await wait(1500);
  await claim(userWallet, airdropContract, [4, 0, 1]);

  await waitUntil(endtime);
  return airdropContract;
}
