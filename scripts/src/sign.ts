import * as secp256k1 from "secp256k1";
import keccak256 from "keccak256";

const PUBLICKEY =
  process.argv[2] ?? "A59iiunFlPQJGnIWvgJlUIcADoSDHZ4ROcZIYhldJfvD";
const PRIVATEKEY =
  process.argv[3] ?? "/QH1Vgg0kk/S0xip2zLyW0uaHFfcYln6N6MmOnoIJBI=";

type ClaimMsg = {
  claim: {
    signature: string;
    claim_amount: string[];
  };
};

function claim(
  privateKey: string,
  claimee: string,
  airdropContract: string,
  amounts: number[]
) {
  const privateKeyBuf = Buffer.from(privateKey ?? "", "base64");
  const claimsContract = airdropContract;
  const amountstr = amounts
    .map((amount) => amount.toLocaleString("us", { useGrouping: false }))
    .join(",");
  const claimstr = claimee + "," + amountstr + "," + claimsContract;
  const msg = keccak256(Buffer.from(claimstr));
  const sigObj = secp256k1.ecdsaSign(msg, privateKeyBuf);
  const signature = Buffer.from(sigObj.signature).toString("base64");
  const claimMsg = {
    claim: {
      signature: signature,
      claim_amount: amount.map((amount) => amount.toString()),
    },
  };
  return claimMsg;
}

function verifyClaim(
  claimee: string,
  claimMsg: ClaimMsg,
  airdropContract: string,
  publicKey: string
) {
  const publicKeyBuf = Buffer.from(publicKey ?? "", "base64");
  const claimsContract = airdropContract;
  const amountstr = claimMsg.claim.claim_amount.join(",");
  const claimstr = claimee + "," + amountstr + "," + claimsContract;
  console.log("claimstr = ", claimstr);
  const msg = keccak256(Buffer.from(claimstr));
  const sig = Buffer.from(claimMsg.claim.signature, "base64");
  const isVerified = secp256k1.ecdsaVerify(sig, msg, publicKeyBuf);
  return isVerified;
}

const contract =
  "terra1tpnkvcd6hm8mtgu2fmrwxux4lgyrs4fvneqjeh04dehw49tekuss6p3dun";
const claimee = "terra19dyjj0te08nsjfljgyc30dmw5mrrmnj0mmcdtq";
const amount = [100000, 20000, 98765];
const claimMsg = claim(PRIVATEKEY, claimee, contract, amount);
console.log("claimMsg = ", claimMsg);
const isVerified = verifyClaim(claimee, claimMsg, contract, PUBLICKEY);
console.log("isVerified = ", isVerified);
