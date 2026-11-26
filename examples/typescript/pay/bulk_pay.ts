import { argv } from "node:process";

import { isEvmAddress, parsePaymentAmount } from "unifi-sdk-ts/primitives";
import type { BulkPaymentItem } from "unifi-sdk-ts/primitives";

import { client, readChain, readCoin, required } from "../shared.ts";

const args = argv.slice(2);
const submit = args.at(-1) === "--submit";
if (submit) args.pop();
const [user, firstAddress, firstAmount, secondAddress, secondAmount, chainName, coinName] = args;
const userId = required(user, "a user ID");
const chain = readChain(chainName);
const coin = readCoin(coinName);

function payment(address: string | undefined, value: string | undefined): BulkPaymentItem {
  const toAddress = required(address, "a recipient address");
  const amount = required(value, "an amount");
  if (!isEvmAddress(toAddress)) throw new Error("Invalid recipient address");
  if (parsePaymentAmount(amount, coin) === 0n) throw new Error("Payment amount must be positive");
  return { to_address: toAddress, coin, amount, memo: "BulkPay" };
}

const batch = [payment(firstAddress, firstAmount), payment(secondAddress, secondAmount)];
const sdk = client();
const quote = await sdk.getManagedBulkPaymentPreflight(userId, chain, [coin]);
console.log(quote.coin_entries);
if (submit) {
  console.log(await sdk.createManagedBulkPayment(userId, false, { chain, batch, save_batch: false }));
}
