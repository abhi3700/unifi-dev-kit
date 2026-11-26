import { argv } from "node:process";

import { client, readCoin, required } from "../shared.ts";

const [, , userId, coin] = argv;
const selectedCoin = readCoin(coin);
await client().requestFaucet(required(userId, "a user ID"), selectedCoin, "Sepolia");
console.log(`Requested ${selectedCoin} on Sepolia`);
