"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (_) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
exports.__esModule = true;
var feather_js_1 = require("@terra-money/feather.js");
var process = require("process");
var fs = require("fs");
var path = require("path");
var secp256k1 = require("secp256k1");
var keccak256_1 = require("keccak256");
console.log("Example usage: yarn start src/test.ts --m1");
var IS_M1 = process.argv[2] === "--m1";
var M1_MODIFIER = IS_M1 ? "-aarch64" : "";
var SEED_PHRASE = process.env.SEED_PHRASE;
var PROTOCOL_PHRASE = process.env.PROTOCOL_PHRASE;
var USER_PHRASE = process.env.USER_PHRASE;
var PUBLICKEY = process.env.PUBLICKEY;
var PRIVATEKEY = process.env.PRIVATEKEY;
var FACTORY_CONTRACT = "genie-airdrop-factory";
var CONTRACT = "genie-airdrop";
var TOKEN_CONTRACT = "terra167dsqkh2alurx997wmycw9ydkyu54gyswe3ygmrs4lwume3vmwks8ruqnv";
var asset_info = {
    token: {
        contract_addr: TOKEN_CONTRACT
    }
};
var asset_info_luna = {
    native_token: {
        denom: "uluna"
    }
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
var terra = new feather_js_1.LCDClient({
    "pisco-1": {
        lcd: "https://pisco-lcd.terra.dev",
        chainID: "pisco-1",
        gasAdjustment: 1.75,
        gasPrices: { uluna: 0.015 },
        prefix: "terra"
    }
});
var key = new feather_js_1.MnemonicKey({
    mnemonic: SEED_PHRASE
});
var hallwallet = terra.wallet(key);
var key2 = new feather_js_1.MnemonicKey({
    mnemonic: PROTOCOL_PHRASE
});
var protocolWallet = terra.wallet(key2);
var key3 = new feather_js_1.MnemonicKey({
    mnemonic: USER_PHRASE
});
var userWallet = terra.wallet(key3);
var factoryFile = fs.readFileSync(path.resolve(__dirname, "..", "..", "artifacts", FACTORY_CONTRACT.replace(/-/g, "_") + M1_MODIFIER + ".wasm"));
var file = fs.readFileSync(path.resolve(__dirname, "..", "..", "artifacts", CONTRACT.replace(/-/g, "_") + M1_MODIFIER + ".wasm"));
function uploadContract(wallet) {
    return __awaiter(this, void 0, void 0, function () {
        var uploadFactory, upload, tx, res, factoryCode, contractCode;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    uploadFactory = new feather_js_1.MsgStoreCode(wallet.key.accAddress("terra"), Buffer.from(factoryFile).toString("base64"));
                    upload = new feather_js_1.MsgStoreCode(wallet.key.accAddress("terra"), Buffer.from(file).toString("base64"));
                    return [4 /*yield*/, wallet.createAndSignTx({
                            msgs: [uploadFactory, upload],
                            chainID: "pisco-1"
                        })];
                case 1:
                    tx = _a.sent();
                    console.log(tx);
                    console.log("----------------------------------");
                    return [4 /*yield*/, terra.tx.broadcast(tx, "pisco-1")];
                case 2:
                    res = _a.sent();
                    console.log(res);
                    factoryCode = parseInt(res.logs[0].events[1].attributes[0].value);
                    contractCode = parseInt(res.logs[1].events[1].attributes[0].value);
                    console.log("factoryCode", factoryCode);
                    console.log("contractCode", contractCode);
                    return [2 /*return*/, [factoryCode, contractCode]];
            }
        });
    });
}
function instantiateFactory(wallet, factoryCode, contractCode) {
    return __awaiter(this, void 0, void 0, function () {
        var initMsg, instantiateFactory, tx, res, factoryContract;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    initMsg = { airdrop_code_id: contractCode, public_key: PUBLICKEY };
                    instantiateFactory = new feather_js_1.MsgInstantiateContract(wallet.key.accAddress("terra"), undefined, factoryCode, initMsg, {}, "factory");
                    return [4 /*yield*/, wallet.createAndSignTx({
                            msgs: [instantiateFactory],
                            chainID: "pisco-1"
                        })];
                case 1:
                    tx = _a.sent();
                    console.log(tx);
                    console.log("----------------------------------");
                    return [4 /*yield*/, terra.tx.broadcast(tx, "pisco-1")];
                case 2:
                    res = _a.sent();
                    console.log(res);
                    factoryContract = res.logs[0].events[0].attributes[0].value;
                    console.log("factoryContract", factoryContract);
                    return [2 /*return*/, factoryContract];
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
                            asset_info: asset,
                            from_timestamp: from_timestamp,
                            to_timestamp: to_timestamp,
                            allocated_amount: allocated_amount.toString()
                        }
                    }, {});
                    return [4 /*yield*/, wallet.createAndSignTx({
                            msgs: [createAirdrop],
                            chainID: "pisco-1"
                        })];
                case 1:
                    tx = _a.sent();
                    console.log(tx);
                    console.log("----------------------------------");
                    return [4 /*yield*/, terra.tx.broadcast(tx, "pisco-1")];
                case 2:
                    res = _a.sent();
                    console.log(res);
                    airdropContract = res.logs[0].events[1].attributes[0].value;
                    console.log("airdropContract", airdropContract);
                    return [2 /*return*/, airdropContract];
            }
        });
    });
}
function increaseIncentives(wallet, token_contract, amount, airdropContract) {
    return __awaiter(this, void 0, void 0, function () {
        var astroSend, sendTokens, tx, res;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    astroSend = {
                        send: {
                            contract: airdropContract,
                            amount: amount.toString(),
                            msg: Buffer.from(JSON.stringify({
                                increase_incentives: {}
                            })).toString("base64")
                        }
                    };
                    sendTokens = new feather_js_1.MsgExecuteContract(wallet.key.accAddress("terra"), token_contract, astroSend, {});
                    return [4 /*yield*/, wallet.createAndSignTx({
                            msgs: [sendTokens],
                            chainID: "pisco-1"
                        })];
                case 1:
                    tx = _a.sent();
                    console.log(tx);
                    console.log("----------------------------------");
                    return [4 /*yield*/, terra.tx.broadcast(tx, "pisco-1")];
                case 2:
                    res = _a.sent();
                    console.log(res);
                    return [2 /*return*/];
            }
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
                    }, { uluna: amount.toString() });
                    return [4 /*yield*/, wallet.createAndSignTx({
                            msgs: [increaseIncentives],
                            chainID: "pisco-1"
                        })];
                case 1:
                    tx = _a.sent();
                    console.log(tx);
                    console.log("----------------------------------");
                    return [4 /*yield*/, terra.tx.broadcast(tx, "pisco-1")];
                case 2:
                    res = _a.sent();
                    console.log(res);
                    return [2 /*return*/];
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
                    account = wallet.key.accAddress("terra");
                    claimsContract = airdropContract;
                    claimstr = account + "," + amount + "," + claimsContract;
                    msg = keccak256_1["default"](Buffer.from(claimstr));
                    sigObj = secp256k1.ecdsaSign(msg, private_key);
                    signature = Buffer.from(sigObj.signature).toString("base64");
                    claim = new feather_js_1.MsgExecuteContract(wallet.key.accAddress("terra"), airdropContract, {
                        claim: {
                            signature: signature,
                            claim_amount: amount.toString()
                        }
                    }, {});
                    return [4 /*yield*/, wallet.createAndSignTx({
                            msgs: [claim],
                            chainID: "pisco-1"
                        })];
                case 1:
                    tx = _a.sent();
                    console.log(tx);
                    console.log("----------------------------------");
                    return [4 /*yield*/, terra.tx.broadcast(tx, "pisco-1")];
                case 2:
                    res = _a.sent();
                    console.log(res);
                    return [2 /*return*/, res];
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
                        transfer_unclaimed_tokens: {
                            amount: amount.toString(),
                            recipient: wallet.key.accAddress("terra")
                        }
                    }, {});
                    return [4 /*yield*/, wallet.createAndSignTx({
                            msgs: [transferUnclaimed],
                            chainID: "pisco-1"
                        })];
                case 1:
                    tx = _a.sent();
                    console.log(tx);
                    console.log("----------------------------------");
                    return [4 /*yield*/, terra.tx.broadcast(tx, "pisco-1")];
                case 2:
                    res = _a.sent();
                    console.log(res);
                    return [2 /*return*/, res];
            }
        });
    });
}
function wait(ms) {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            return [2 /*return*/, new Promise(function (resolve) {
                    setTimeout(resolve, ms);
                })];
        });
    });
}
function waitUntil(s) {
    return __awaiter(this, void 0, void 0, function () {
        var timeUntil, waiting_time;
        return __generator(this, function (_a) {
            timeUntil = s * 1000;
            console.log("timeUntil", timeUntil);
            waiting_time = timeUntil - Date.now() > 0 ? timeUntil - Date.now() + 60000 : 60000;
            // try to wait until RPC/LCD and feather.js is updated with a time in the future
            // takes at least 1 minute to update properly
            console.log("waiting_time", waiting_time);
            return [2 /*return*/, new Promise(function (resolve) {
                    setTimeout(resolve, waiting_time);
                })];
        });
    });
}
testall();
function testall() {
    return __awaiter(this, void 0, void 0, function () {
        var factoryContract;
        var _this = this;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    console.log("UPLOADING CONTRACTS");
                    return [4 /*yield*/, uploadContract(hallwallet).then(function (res) { return __awaiter(_this, void 0, void 0, function () {
                            return __generator(this, function (_a) {
                                switch (_a.label) {
                                    case 0: return [4 /*yield*/, wait(6000)];
                                    case 1:
                                        _a.sent();
                                        console.log("INSTANTIATING FACTORY");
                                        return [2 /*return*/, instantiateFactory(hallwallet, res[0], res[1])];
                                }
                            });
                        }); })];
                case 1:
                    factoryContract = _a.sent();
                    // const factoryContract =
                    //   "terra13yd2sp8m9w92c35djuuy08856wflgum95cweq9zaxlwsvtyqc7hss68m26";
                    return [4 /*yield*/, wait(6000)];
                case 2:
                    // const factoryContract =
                    //   "terra13yd2sp8m9w92c35djuuy08856wflgum95cweq9zaxlwsvtyqc7hss68m26";
                    _a.sent();
                    console.log("TESTING TEST 1");
                    return [4 /*yield*/, test1(factoryContract)["catch"](function (err) {
                            console.log(err);
                        })];
                case 3:
                    _a.sent();
                    return [4 /*yield*/, wait(6000)];
                case 4:
                    _a.sent();
                    console.log("TESTING TEST 2");
                    return [4 /*yield*/, test2(factoryContract)["catch"](function (err) {
                            console.log(err);
                        })];
                case 5:
                    _a.sent();
                    return [4 /*yield*/, wait(20000)];
                case 6:
                    _a.sent();
                    console.log("TESTING TEST 3");
                    return [4 /*yield*/, test3(factoryContract)["catch"](function (err) {
                            console.log(err);
                        })];
                case 7:
                    _a.sent();
                    console.log("DONE TESTING");
                    return [2 /*return*/];
            }
        });
    });
}
function test1(factoryContract) {
    return __awaiter(this, void 0, void 0, function () {
        var starttime, endtime, airdropContract;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    starttime = Math.trunc(Date.now() / 1000 + 60);
                    endtime = Math.trunc(Date.now() / 1000 + 500);
                    return [4 /*yield*/, wait(6000)];
                case 1:
                    _a.sent();
                    return [4 /*yield*/, createAirdrop(protocolWallet, factoryContract, asset_info, 5000000, starttime, endtime)];
                case 2:
                    airdropContract = _a.sent();
                    return [4 /*yield*/, wait(6000)];
                case 3:
                    _a.sent();
                    return [4 /*yield*/, increaseIncentives(protocolWallet, TOKEN_CONTRACT, 2000000, airdropContract)];
                case 4:
                    _a.sent();
                    return [4 /*yield*/, increaseIncentives(protocolWallet, TOKEN_CONTRACT, 7000000, airdropContract)];
                case 5:
                    _a.sent();
                    return [4 /*yield*/, wait(6000)];
                case 6:
                    _a.sent();
                    return [4 /*yield*/, transferUnclaimedTokens(protocolWallet, airdropContract, 4000000)];
                case 7:
                    _a.sent();
                    return [4 /*yield*/, waitUntil(starttime)];
                case 8:
                    _a.sent();
                    return [4 /*yield*/, claim(userWallet, airdropContract, 2000000)];
                case 9:
                    _a.sent();
                    return [4 /*yield*/, wait(6000)];
                case 10:
                    _a.sent(); // Wait a bit for wallet nonce to update.
                    return [4 /*yield*/, claim(userWallet, airdropContract, 2000000)
                            .then(function (_) {
                            throw new Error("Error not thrown");
                        })["catch"](function (err) {
                            if (err.message !== "Error not thrown") {
                                console.log("error is thrown for double claim");
                            }
                            else {
                                throw new Error("Error not thrown");
                            }
                        })];
                case 11:
                    _a.sent();
                    return [4 /*yield*/, claim(protocolWallet, airdropContract, 4000000)];
                case 12:
                    _a.sent();
                    return [4 /*yield*/, claim(hallwallet, airdropContract, 1000000)
                            .then(function (_) {
                            throw new Error("Error not thrown");
                        })["catch"](function (err) {
                            if (err.message !== "Error not thrown") {
                                console.log("error is thrown for double claim");
                            }
                            else {
                                throw new Error("Error not thrown");
                            }
                        })];
                case 13:
                    _a.sent();
                    return [2 /*return*/, airdropContract];
            }
        });
    });
}
function test2(factoryContract) {
    return __awaiter(this, void 0, void 0, function () {
        var starttime, endtime, airdropContract;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    starttime = Math.trunc(Date.now() / 1000 + 60);
                    endtime = Math.trunc(Date.now() / 1000 + 600);
                    return [4 /*yield*/, createAirdrop(protocolWallet, factoryContract, asset_info_luna, 5000000, starttime, endtime)];
                case 1:
                    airdropContract = _a.sent();
                    wait(6000);
                    return [4 /*yield*/, increaseLunaIncentives(protocolWallet, airdropContract, 2000000)];
                case 2:
                    _a.sent();
                    return [4 /*yield*/, claim(userWallet, airdropContract, 2000000)
                            .then(function (_) {
                            throw new Error("Error not thrown");
                        })["catch"](function (err) {
                            if (err.message !== "Error not thrown") {
                                console.log("Error is thrown for claim before start");
                            }
                            else {
                                throw new Error("Error not thrown");
                            }
                        })];
                case 3:
                    _a.sent();
                    return [4 /*yield*/, waitUntil(starttime)];
                case 4:
                    _a.sent();
                    return [4 /*yield*/, claim(userWallet, airdropContract, 2000000)
                            .then(function (_) {
                            throw new Error("Error not thrown");
                        })["catch"](function (err) {
                            if (err.message !== "Error not thrown") {
                                console.log("Error is thrown for being unable to claim due to no more tokens");
                            }
                            else {
                                throw new Error("Error not thrown");
                            }
                        })];
                case 5:
                    _a.sent();
                    transferUnclaimedTokens(protocolWallet, airdropContract, 2000000);
                    return [2 /*return*/];
            }
        });
    });
}
function test3(factoryContract) {
    return __awaiter(this, void 0, void 0, function () {
        var starttime, endtime, airdropContract;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    starttime = Math.trunc(Date.now() / 1000 + 50);
                    endtime = Math.trunc(Date.now() / 1000 + 200);
                    return [4 /*yield*/, createAirdrop(protocolWallet, factoryContract, asset_info, 5000000, starttime, endtime)];
                case 1:
                    airdropContract = _a.sent();
                    return [4 /*yield*/, wait(6000)];
                case 2:
                    _a.sent();
                    return [4 /*yield*/, increaseIncentives(protocolWallet, TOKEN_CONTRACT, 6000000, airdropContract)];
                case 3:
                    _a.sent();
                    return [4 /*yield*/, waitUntil(starttime)];
                case 4:
                    _a.sent();
                    return [4 /*yield*/, claim(userWallet, airdropContract, 2000000)];
                case 5:
                    _a.sent();
                    return [4 /*yield*/, waitUntil(endtime)];
                case 6:
                    _a.sent();
                    return [4 /*yield*/, transferUnclaimedTokens(protocolWallet, airdropContract, 4000000)];
                case 7:
                    _a.sent();
                    return [2 /*return*/];
            }
        });
    });
}
