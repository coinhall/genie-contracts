import { randomBytes } from "crypto";
import * as secp256k1 from "secp256k1";

const privKey = randomBytes(32);
if (!secp256k1.privateKeyVerify(privKey)) {
  console.error("Private key is not valid!");
  process.exit(1);
}
const pubKey = secp256k1.publicKeyCreate(privKey);
if (!secp256k1.publicKeyVerify(pubKey)) {
  console.error("Public key is not valid!");
  process.exit(1);
}

console.log("Public key: ", Buffer.from(pubKey).toString("base64"));
console.log("Private key:", Buffer.from(privKey).toString("base64"));
