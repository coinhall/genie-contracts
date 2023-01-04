import { randomBytes } from "crypto";
import * as secp256k1 from "secp256k1";

let privKey;
do {
  console.log("Generating private key...");
  privKey = randomBytes(32);
} while (!secp256k1.privateKeyVerify(privKey));

// get the public key in a compressed format
const pubKey = secp256k1.publicKeyCreate(privKey);

console.log("PUBLICKEY = ", Buffer.from(pubKey).toString("base64"));
console.log("PRIVATEKEY = ", privKey.toString("base64"));
