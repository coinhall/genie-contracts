import {
  LCDClient,
  MnemonicKey,
  MsgExecuteContract,
} from "@terra-money/feather.js";
import dotenv from "dotenv";
import * as readline from "node:readline/promises";
import { stdin as input, stdout as output } from "node:process";

const PREFIX = "terra";
const CHAIN_ID = "pisco-1";

const terra = new LCDClient({
  "pisco-1": {
    lcd: "https://pisco-lcd.terra.dev",
    chainID: CHAIN_ID,
    gasAdjustment: 1.75,
    gasPrices: { uluna: 0.15 },
    prefix: PREFIX,
  },
});

(async () => {
  dotenv.config();
  const protocolSeedPhrase = process.env.PROTOCOL_PHRASE;
  if (protocolSeedPhrase == null) {
    throw Error("Please input 'PROTOCOL_PHRASE' in config/.env");
  }
  const rl = readline.createInterface({ input, output });
  const factoryAddress = await rl.question("Factory address: "); // e.g. terra18kqdqsnq924es3xldjr707ygcg4859gt6juevkcelk087efw5jjq36gre2
  const nftAddress = await rl.question("NFT address: "); // e.g. terra1fj3zm6rm4ejuccvkfjcu88hktv5evs6ehsuazas3ckhj0g39lnesaphhma
  const amountOfMissions = Number(await rl.question("Number of missions: "));
  const budgets: string[] = [];
  for (let i = 0; i < amountOfMissions; i++) {
    budgets.push(await rl.question(`Budget for mission ${i + 1}: `));
  }

  const key = new MnemonicKey({
    mnemonic: protocolSeedPhrase,
  });
  const protocolWallet = terra.wallet(key);

  const timeNow = Math.floor(new Date().getTime() / 1000);
  const timeIn10Mins = timeNow + 10 * 60;
  const timeIn20Mins = timeIn10Mins + 10 * 60;

  const createNftAirdrop = new MsgExecuteContract(
    protocolWallet.key.accAddress(PREFIX),
    factoryAddress,
    {
      create_nft_airdrop: {
        nft_info: {
          contract_addr: nftAddress,
        },
        from_timestamp: timeIn10Mins,
        to_timestamp: timeIn20Mins,
        allocated_amounts: budgets,
        campaign_id: new Date().toUTCString(), // just use unixtimestamp for unique identifier
        icon_url:
          "https://i.seadn.io/gae/H8jOCJuQokNqGBpkBN5wk1oZwO7LM8bNnrHCaekV2nKjnCqw6UB5oaH8XyNeBDj6bA_n1mjejzhFQUP3O1NfjFLHr3FOaeHcTOOT?auto=format&dpr=1&w=256", // random azuki icon
      },
    },
    {}
  );
  const tx = await protocolWallet.createAndSignTx({
    msgs: [createNftAirdrop],
    chainID: CHAIN_ID,
  });
  console.log("----------------------------------");
  const res = await terra.tx.broadcast(tx, CHAIN_ID);
  console.log(res);
  const airdropContract = res.logs[0].events
    .find((x) => x.type === "instantiate")
    ?.attributes.find((x) => x.key == "_contract_address")
    ?.value.toString();
  if (!airdropContract) {
    throw new Error("Airdrop contract not found!!");
  }
  console.log("\x1b[32mContract Address: %s\x1b[0m", airdropContract);
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
