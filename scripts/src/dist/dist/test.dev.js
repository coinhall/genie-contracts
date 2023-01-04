"use strict";

var __awaiter = void 0 && (void 0).__awaiter || function (thisArg, _arguments, P, generator) {
  function adopt(value) {
    return value instanceof P ? value : new P(function (resolve) {
      resolve(value);
    });
  }

  return new (P || (P = Promise))(function (resolve, reject) {
    function fulfilled(value) {
      try {
        step(generator.next(value));
      } catch (e) {
        reject(e);
      }
    }

    function rejected(value) {
      try {
        step(generator["throw"](value));
      } catch (e) {
        reject(e);
      }
    }

    function step(result) {
      result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected);
    }

    step((generator = generator.apply(thisArg, _arguments || [])).next());
  });
};

var __generator = void 0 && (void 0).__generator || function (thisArg, body) {
  var _ = {
    label: 0,
    sent: function sent() {
      if (t[0] & 1) throw t[1];
      return t[1];
    },
    trys: [],
    ops: []
  },
      f,
      y,
      t,
      g;
  return g = {
    next: verb(0),
    "throw": verb(1),
    "return": verb(2)
  }, typeof Symbol === "function" && (g[Symbol.iterator] = function () {
    return this;
  }), g;

  function verb(n) {
    return function (v) {
      return step([n, v]);
    };
  }

  function step(op) {
    if (f) throw new TypeError("Generator is already executing.");

    while (_) {
      try {
        if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
        if (y = 0, t) op = [op[0] & 2, t.value];

        switch (op[0]) {
          case 0:
          case 1:
            t = op;
            break;

          case 4:
            _.label++;
            return {
              value: op[1],
              done: false
            };

          case 5:
            _.label++;
            y = op[1];
            op = [0];
            continue;

          case 7:
            op = _.ops.pop();

            _.trys.pop();

            continue;

          default:
            if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) {
              _ = 0;
              continue;
            }

            if (op[0] === 3 && (!t || op[1] > t[0] && op[1] < t[3])) {
              _.label = op[1];
              break;
            }

            if (op[0] === 6 && _.label < t[1]) {
              _.label = t[1];
              t = op;
              break;
            }

            if (t && _.label < t[2]) {
              _.label = t[2];

              _.ops.push(op);

              break;
            }

            if (t[2]) _.ops.pop();

            _.trys.pop();

            continue;
        }

        op = body.call(thisArg, _);
      } catch (e) {
        op = [6, e];
        y = 0;
      } finally {
        f = t = 0;
      }
    }

    if (op[0] & 5) throw op[1];
    return {
      value: op[0] ? op[1] : void 0,
      done: true
    };
  }
};

var _a, _b, _c;

exports.__esModule = true;

var feather_js_1 = require("@terra-money/feather.js");

var process = require("process");

var fs = require("fs");

var path = require("path");

var secp256k1 = require("secp256k1");

var keccak256_1 = require("keccak256");

console.log("Example usage: yarn start src/test.ts --mainnet");
var SEED_PHRASE = process.env.SEED_PHRASE;
var PROTOCOL_PHRASE = process.env.PROTOCOL_PHRASE;
var USER_PHRASE = process.env.USER_PHRASE;
var PUBLICKEY = process.env.PUBLICKEY;
var PRIVATEKEY = process.env.PRIVATEKEY; // const IS_MAINNET = process.argv[2] === "--mainnet";

var FACTORY_CONTRACT = "genie-factory-v1-terra2";
var CONTRACT = "genie-v1-terra2";
var TOKEN_CONTRACT = "terra167dsqkh2alurx997wmycw9ydkyu54gyswe3ygmrs4lwume3vmwks8ruqnv";
var ASSET = {
  token: {
    contract_addr: TOKEN_CONTRACT
  }
};
var TO_TIMESTAMP = 1933046400;

if (!SEED_PHRASE) {
  console.log("Missing SEED_PHRASE env var");
  process.exit(1);
}

if (!PROTOCOL_PHRASE) {
  console.log("Missing PROTOCOL_PHRASE env var");
  process.exit(1);
}

if (!USER_PHRASE) {
  console.log("Missing PROTOCOL_PHRASE env var");
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

var terra = new feather_js_1.LCDClient({
  // key must be the chainID
  "pisco-1": {
    lcd: "https://pisco-lcd.terra.dev",
    chainID: "pisco-1",
    gasAdjustment: 1.75,
    gasPrices: {
      uluna: 0.015
    },
    prefix: "terra"
  }
});
var key = new feather_js_1.MnemonicKey({
  mnemonic: SEED_PHRASE
});
var wallet = terra.wallet(key);
var key2 = new feather_js_1.MnemonicKey({
  mnemonic: PROTOCOL_PHRASE
});
var protocolWallet = terra.wallet(key2);
var key3 = new feather_js_1.MnemonicKey({
  mnemonic: USER_PHRASE
});
var userWallet = terra.wallet(key3);
var walletAddress = (_a = wallet.key.accAddress("terra")) !== null && _a !== void 0 ? _a : "";
var protocolAddress = (_b = protocolWallet.key.accAddress("terra")) !== null && _b !== void 0 ? _b : "";
var userAddress = (_c = userWallet.key.accAddress("terra")) !== null && _c !== void 0 ? _c : "";
var factoryFile = fs.readFileSync(path.resolve(__dirname, "..", "..", "artifacts", FACTORY_CONTRACT.replace(/-/g, "_") + "-aarch64" + ".wasm"));
var file = fs.readFileSync(path.resolve(__dirname, "..", "..", "..", CONTRACT, "artifacts", CONTRACT.replace(/-/g, "_") + "-aarch64" + ".wasm"));

function uploadContract() {
  return __awaiter(this, void 0, void 0, function () {
    var uploadFactory, upload, tx, res, factoryCode, contractCode;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          uploadFactory = new feather_js_1.MsgStoreCode(walletAddress, Buffer.from(factoryFile).toString("base64"));
          upload = new feather_js_1.MsgStoreCode(walletAddress, Buffer.from(file).toString("base64"));
          return [4
          /*yield*/
          , wallet.createAndSignTx({
            msgs: [uploadFactory, upload],
            chainID: "pisco-1"
          })];

        case 1:
          tx = _a.sent();
          console.log(tx);
          console.log("----------------------------------");
          return [4
          /*yield*/
          , terra.tx.broadcast(tx, "pisco-1")];

        case 2:
          res = _a.sent();
          console.log(res);
          factoryCode = parseInt(res.logs[0].events[1].attributes[0].value);
          contractCode = parseInt(res.logs[1].events[1].attributes[0].value);
          console.log("factoryCode", factoryCode);
          console.log("contractCode", contractCode);
          return [2
          /*return*/
          , [factoryCode, contractCode]];
      }
    });
  });
}

function instantiateFactory(factoryCode, contractCode) {
  return __awaiter(this, void 0, void 0, function () {
    var initMsg, instantiateFactory, tx, res, factoryContract;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          initMsg = {
            airdrop_code_id: contractCode,
            public_key: PUBLICKEY
          };
          instantiateFactory = new feather_js_1.MsgInstantiateContract(walletAddress, undefined, factoryCode, initMsg, {}, "factory");
          return [4
          /*yield*/
          , wallet.createAndSignTx({
            msgs: [instantiateFactory],
            chainID: "pisco-1"
          })];

        case 1:
          tx = _a.sent();
          console.log(tx);
          console.log("----------------------------------");
          return [4
          /*yield*/
          , terra.tx.broadcast(tx, "pisco-1")];

        case 2:
          res = _a.sent();
          console.log(res);
          factoryContract = res.logs[0].events[1].attributes[0].value;
          console.log("factoryContract", factoryContract);
          return [2
          /*return*/
          , factoryContract];
      }
    });
  });
}

function createAirdrop(wallet, factoryContract, asset, allocated_amount, from_timestamp, to_timestamp) {
  return __awaiter(this, void 0, void 0, function () {
    var createAirdrop, tx, res, airdropContract;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          createAirdrop = new feather_js_1.MsgExecuteContract(wallet.key.accAddress("terra"), factoryContract, {
            create_airdrop: {
              asset: asset,
              to_timestamp: to_timestamp.toString(),
              from_timestamp: from_timestamp.toString,
              allocated_amount: allocated_amount
            }
          }, {});
          return [4
          /*yield*/
          , wallet.createAndSignTx({
            msgs: [createAirdrop],
            chainID: "pisco-1"
          })];

        case 1:
          tx = _a.sent();
          console.log(tx);
          console.log("----------------------------------");
          return [4
          /*yield*/
          , terra.tx.broadcast(tx, "pisco-1")];

        case 2:
          res = _a.sent();
          console.log(res);
          airdropContract = res.logs[0].events[1].attributes[0].value;
          console.log("airdropContract", airdropContract);
          return [2
          /*return*/
          , airdropContract];
      }
    });
  });
}

function increaseIncentives(wallet, asset, amount, airdropContract) {
  return __awaiter(this, void 0, void 0, function () {
    var astroSend, sendTokens;
    return __generator(this, function (_a) {
      astroSend = {
        send: {
          recipient: airdropContract,
          amount: amount.toString()
        }
      };
      sendTokens = new feather_js_1.MsgExecuteContract(wallet.key.accAddress("terra"), TOKEN_CONTRACT, astroSend, {});
      return [2
      /*return*/
      , wallet.createAndSignTx({
        msgs: [sendTokens],
        chainID: "pisco-1"
      })];
    });
  });
}

function increaseLunaIncentives(wallet, airdropContract, amount) {
  return __awaiter(this, void 0, void 0, function () {
    var increaseIncentives, tx, res;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          increaseIncentives = new feather_js_1.MsgExecuteContract(wallet.key.accAddress("terra"), airdropContract, {
            increase_incentives: {}
          }, {
            amount: amount
          });
          return [4
          /*yield*/
          , wallet.createAndSignTx({
            msgs: [increaseIncentives],
            chainID: "pisco-1"
          })];

        case 1:
          tx = _a.sent();
          console.log(tx);
          console.log("----------------------------------");
          return [4
          /*yield*/
          , terra.tx.broadcast(tx, "pisco-1")];

        case 2:
          res = _a.sent();
          console.log(res);
          return [2
          /*return*/
          ];
      }
    });
  });
}

function claim(wallet, airdropContract, amount) {
  return __awaiter(this, void 0, void 0, function () {
    var private_key, account, claimsContract, claimstr, msg, sigObj, signature, claim, tx, res;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          private_key = Buffer.from(PRIVATEKEY !== null && PRIVATEKEY !== void 0 ? PRIVATEKEY : "", "base64");
          account = userAddress;
          claimsContract = airdropContract;
          claimstr = account + "," + amount + "," + claimsContract;
          msg = keccak256_1["default"](Buffer.from(claimstr));
          sigObj = secp256k1.ecdsaSign(msg, private_key);
          signature = Buffer.from(sigObj.signature).toString("base64");
          claim = new feather_js_1.MsgExecuteContract(wallet.key.accAddress("terra"), airdropContract, {
            claim: {
              signature: signature,
              amount: amount.toString()
            }
          }, {
            amount: amount
          });
          return [4
          /*yield*/
          , wallet.createAndSignTx({
            msgs: [claim],
            chainID: "pisco-1"
          })];

        case 1:
          tx = _a.sent();
          console.log(tx);
          console.log("----------------------------------");
          return [4
          /*yield*/
          , terra.tx.broadcast(tx, "pisco-1")];

        case 2:
          res = _a.sent();
          console.log(res);
          return [2
          /*return*/
          , res];
      }
    });
  });
}

function transferUnclaimedTokens(wallet, airdropContract, amount) {
  return __awaiter(this, void 0, void 0, function () {
    var transferUnclaimed, tx, res;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          transferUnclaimed = new feather_js_1.MsgExecuteContract(wallet.key.accAddress("terra"), airdropContract, {
            transfer_unclaimed: {
              amount: amount.toString(),
              recipient: wallet.key.accAddress("terra")
            }
          }, {});
          return [4
          /*yield*/
          , wallet.createAndSignTx({
            msgs: [transferUnclaimed],
            chainID: "pisco-1"
          })];

        case 1:
          tx = _a.sent();
          console.log(tx);
          console.log("----------------------------------");
          return [4
          /*yield*/
          , terra.tx.broadcast(tx, "pisco-1")];

        case 2:
          res = _a.sent();
          console.log(res);
          return [2
          /*return*/
          , res];
      }
    });
  });
}

test1().then(function (res) {
  return console.log(res);
})["catch"](function (err) {
  return console.log(err);
});

function test1() {
  return __awaiter(this, void 0, void 0, function () {
    var factoryContract, starttime, endtime, airdropContract, err_1;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          return [4
          /*yield*/
          , uploadContract().then(function (res) {
            return instantiateFactory(res[0], res[1]);
          })];

        case 1:
          factoryContract = _a.sent();
          console.log("TESTING TEST 1");
          starttime = Math.trunc(Date.now() / 1000 + 30);
          endtime = Math.trunc(Date.now() / 1000 + 500);
          return [4
          /*yield*/
          , createAirdrop(protocolWallet, factoryContract, ASSET, 5000000, starttime, endtime)];

        case 2:
          airdropContract = _a.sent();
          return [4
          /*yield*/
          , increaseIncentives(protocolWallet, ASSET, 2000000, airdropContract)];

        case 3:
          _a.sent();

          return [4
          /*yield*/
          , increaseIncentives(protocolWallet, ASSET, 3000000, airdropContract)];

        case 4:
          _a.sent();

          _a.label = 5;

        case 5:
          _a.trys.push([5, 7,, 8]);

          return [4
          /*yield*/
          , transferUnclaimedTokens(protocolWallet, airdropContract, 5000000)];

        case 6:
          _a.sent();

          throw new Error("Error not thrown");

        case 7:
          err_1 = _a.sent();
          if (err_1.message !== "Error not thrown") throw err_1;
          return [3
          /*break*/
          , 8];

        case 8:
          return [4
          /*yield*/
          , new Promise(function (resolve) {
            setTimeout(resolve, starttime - Date.now() > 0 ? starttime - Date.now() : 0);
          })];

        case 9:
          _a.sent();

          return [4
          /*yield*/
          , claim(userWallet, airdropContract, 2000000)];

        case 10:
          _a.sent();

          return [4
          /*yield*/
          , claim(userWallet, airdropContract, 2000000)];

        case 11:
          _a.sent();

          return [4
          /*yield*/
          , claim(protocolWallet, airdropContract, 4000000)];

        case 12:
          _a.sent();

          return [2
          /*return*/
          , airdropContract];
      }
    });
  });
}

function test2() {
  return __awaiter(this, void 0, void 0, function () {
    var starttime, endtime, airdropContract, err_2, err_3;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          starttime = Math.trunc(Date.now() / 1000 + 50);
          endtime = Math.trunc(Date.now() / 1000 + 100);
          return [4
          /*yield*/
          , createAirdrop(protocolWallet, factoryContract, ASSET, 5000000, starttime, endtime)];

        case 1:
          airdropContract = _a.sent();
          return [4
          /*yield*/
          , increaseIncentives(protocolWallet, ASSET, 2000000, airdropContract)];

        case 2:
          _a.sent();

          _a.label = 3;

        case 3:
          _a.trys.push([3, 5,, 6]);

          return [4
          /*yield*/
          , claim(userWallet, airdropContract, 2000000)];

        case 4:
          _a.sent();

          throw new Error("Error not thrown");

        case 5:
          err_2 = _a.sent();
          if (err_2.message !== "Error not thrown") throw err_2;
          return [3
          /*break*/
          , 6];

        case 6:
          return [4
          /*yield*/
          , new Promise(function (resolve) {
            setTimeout(resolve, starttime * 1000 - Date.now() > 0 ? starttime * 1000 - Date.now() : 0);
          })];

        case 7:
          _a.sent();

          _a.label = 8;

        case 8:
          _a.trys.push([8, 10,, 11]);

          return [4
          /*yield*/
          , claim(userWallet, airdropContract, 2000000)];

        case 9:
          _a.sent();

          throw new Error("Error not thrown");

        case 10:
          err_3 = _a.sent();
          if (err_3.message !== "Error not thrown") throw err_3;
          return [3
          /*break*/
          , 11];

        case 11:
          transferUnclaimedTokens(protocolWallet, airdropContract, 5000000);
          return [2
          /*return*/
          ];
      }
    });
  });
}

function test3() {
  return __awaiter(this, void 0, void 0, function () {
    var starttime, endtime, airdropContract;
    return __generator(this, function (_a) {
      switch (_a.label) {
        case 0:
          starttime = Math.trunc(Date.now() / 1000 + 50);
          endtime = Math.trunc(Date.now() / 1000 + 100);
          return [4
          /*yield*/
          , createAirdrop(protocolWallet, factoryContract, ASSET, 5000000, starttime, endtime)];

        case 1:
          airdropContract = _a.sent();
          return [4
          /*yield*/
          , increaseIncentives(protocolWallet, ASSET, 6000000, airdropContract)];

        case 2:
          _a.sent();

          return [4
          /*yield*/
          , new Promise(function (resolve) {
            setTimeout(resolve, starttime * 1000 - Date.now() > 0 ? starttime * 1000 - Date.now() : 0);
          })];

        case 3:
          _a.sent();

          return [4
          /*yield*/
          , claim(userWallet, airdropContract, 2000000)];

        case 4:
          _a.sent();

          return [4
          /*yield*/
          , new Promise(function (resolve) {
            setTimeout(resolve, endtime * 1000 - Date.now() > 0 ? endtime * 1000 - Date.now() + 1 : 0);
          })];

        case 5:
          _a.sent();

          return [4
          /*yield*/
          , transferUnclaimedTokens(protocolWallet, airdropContract, 5000000)];

        case 6:
          _a.sent();

          return [2
          /*return*/
          ];
      }
    });
  });
} // const config = new AccessConfig(2, walletAddress);
// const uploadFactory = new MsgStoreCode(
//   walletAddress,
//   Buffer.from(factoryFile).toString("base64")
// );
// const upload = new MsgStoreCode(
//   walletAddress,
//   Buffer.from(file).toString("base64"),
//   config
// );
// wallet
//   .createAndSignTx({
//     msgs: [uploadFactory, upload],
//     chainID: "pisco-1",
//   })
//   .then((tx) => {
//     console.log(tx);
//     console.log("----------------------------------");
//     return terra.tx.broadcast(tx, "pisco-1");
//   })
//   .then((res) => {
//     console.log(res);
//     const factoryCode = parseInt(res.logs[0].events[1].attributes[0].value);
//     const contractCode = parseInt(res.logs[1].events[1].attributes[0].value);
//     console.log("factoryCode", factoryCode);
//     console.log("contractCode", contractCode);
//     return [factoryCode, contractCode];
//   })
//   .then(([factoryCode, contractCode]) => {
//     const initMsg = { airdrop_code_id: contractCode, public_key: PUBLICKEY };
//     const instantiateFactory = new MsgInstantiateContract(
//       walletAddress,
//       undefined,
//       factoryCode,
//       initMsg,
//       {},
//       "factory"
//     );
//     return wallet.createAndSignTx({
//       msgs: [instantiateFactory],
//       chainID: "pisco-1",
//     });
//   })
//   .then((tx) => {
//     console.log(tx);
//     console.log("----------------------------------");
//     return terra.tx.broadcast(tx, "pisco-1");
//   })
//   .then((res) => {
//     console.log(res);
//     const factoryContract = res.logs[0].events[0].attributes[0].value;
//     console.log("factoryContract", factoryContract);
//     return factoryContract;
//   })
//   .then((factoryContract) => {
//     const createAirdropMessage = {
//       create_airdrop: {
//         asset_info: ASSET,
//         to_timestamp: TO_TIMESTAMP,
//         from_timestamp: Math.trunc(Date.now() / 1000),
//         allocated_amount: "5000000",
//       },
//     };
//     const createAirdrop = new MsgExecuteContract(
//       protocolAddress,
//       factoryContract,
//       createAirdropMessage,
//       {}
//     );
//     return protocolWallet.createAndSignTx({
//       msgs: [createAirdrop],
//       chainID: "pisco-1",
//     });
//   })
//   .then((tx) => {
//     console.log(tx);
//     console.log("----------------------------------");
//     return terra.tx.broadcast(tx, "pisco-1");
//   })
//   .then((tx) => {
//     console.log(tx);
//     const airdropContract = tx.logs[0].events[1].attributes[0].value;
//     console.log(airdropContract);
//     return airdropContract;
//   })
//   .then((airdropContract) => {
//     const astroSend = {
//       send: {
//         recipient: airdropContract,
//         amount: "5000000",
//       },
//     };
//     const sendTokens = new MsgExecuteContract(
//       protocolAddress,
//       TOKEN_CONTRACT,
//       astroSend,
//       {}
//     );
//     return protocolWallet.createAndSignTx({
//       msgs: [sendTokens],
//       chainID: "pisco-1",
//     });
//   })
//   .then((tx) => {
//     console.log(tx);
//     console.log("----------------------------------");
//     return terra.tx.broadcast(tx, "pisco-1");
//   })
//   .then((res) => {
//     console.log(res);
//     console.log("5 tokens sent to airdrop contract");
//     return res.logs[0].events[2].attributes[3].value;
//   })
//   .then((airdropContract) => {
//     const private_key = Buffer.from(PRIVATEKEY, "base64");
//     const account = userAddress;
//     const claimsContract = airdropContract;
//     const amount = 5000000;
//     const claim = account + "," + amount + "," + claimsContract;
//     const msg = keccak256(Buffer.from(claim));
//     const sigObj = secp256k1.ecdsaSign(msg, private_key);
//     const signature = Buffer.from(sigObj.signature).toString("base64");
//     const claim_info = {
//       claim: {
//         claim_amount: amount.toString(),
//         signature: signature,
//       },
//     };
//     const claimMsg = new MsgExecuteContract(
//       userAddress,
//       claimsContract,
//       claim_info,
//       {}
//     );
//     return userWallet.createAndSignTx({
//       msgs: [claimMsg],
//       chainID: "pisco-1",
//     });
//   })
//   .then((tx) => {
//     console.log(tx);
//     console.log("----------------------------------");
//     return terra.tx.broadcast(tx, "pisco-1");
//   })
//   .then((res) => {
//     console.log(res);
//   })
//   .catch((err) => {
//     console.log(err);
//   });