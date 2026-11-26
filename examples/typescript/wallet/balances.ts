import { argv } from "node:process";

import { client, readChain, readCoin, required } from "../shared.ts";

const [, , user, chainName, coinName] = argv;
const userId = required(user, "a user ID");
const sdk = client();
if (chainName === undefined) {
  console.log(await sdk.getWalletBalances(userId));
} else {
  const chain = readChain(chainName);
  if (coinName === undefined) {
    console.log(await sdk.getWalletBalancesByChain(userId, chain));
  } else {
    console.log(await sdk.getWalletBalanceByPairWithFinality(userId, chain, readCoin(coinName)));
  }
}
