"use strict";
exports.__esModule = true;
var crypto_1 = require("crypto");
var secp256k1 = require("secp256k1");
var privKey;
do {
    console.log("Generating private key...");
    privKey = crypto_1.randomBytes(32);
} while (!secp256k1.privateKeyVerify(privKey));
// get the public key in a compressed format
var pubKey = secp256k1.publicKeyCreate(privKey);
console.log("PUBLICKEY = ", Buffer.from(pubKey).toString("base64"));
console.log("PRIVATEKEY = ", privKey.toString("base64"));
