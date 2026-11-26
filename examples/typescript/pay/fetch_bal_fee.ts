import { argv } from "node:process";

import { client, readChain, readCoin, required } from "../shared.ts";

const [, , userId, chain, coin] = argv;
const quote = await client().getPaymentPreflight(
  required(userId, "a user ID"), readChain(chain), readCoin(coin),
);
console.log({ balance: quote.balance, estimatedFee: quote.est_fee, needsApproval: quote.is_coin_allowance_zero });
