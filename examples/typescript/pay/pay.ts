import { argv } from "node:process";

import { isEvmAddress, parsePaymentAmount } from "unifi-sdk-ts/primitives";

import { client, readChain, readCoin, required } from "../shared.ts";

const args = argv.slice(2);
const submit = args.at(-1) === "--submit";
if (submit) args.pop();
const [user, recipient, inputAmount, chainName, coinName] = args;
const userId = required(user, "a user ID");
const toAddress = required(recipient, "a recipient address");
const amount = required(inputAmount, "an amount");
const chain = readChain(chainName);
const coin = readCoin(coinName);

if (!isEvmAddress(toAddress)) throw new Error("Invalid recipient address");
const units = parsePaymentAmount(amount, coin);
if (units === 0n) throw new Error("Payment amount must be positive");

const sdk = client();
const quote = await sdk.getPaymentPreflight(userId, chain, coin);
console.log({ balance: quote.balance, estimatedFee: quote.est_fee });
if (submit) {
  const total = units + parsePaymentAmount(quote.est_fee, coin);
  if (total > parsePaymentAmount(quote.balance, coin)) throw new Error("Insufficient balance for amount and fee");
  const receiptId = await sdk.createManagedWalletPayment(userId, false, {
    payload: { chain, coin, to_address: toAddress, amount, memo: "General" },
  });
  console.log({ receiptId });
}
