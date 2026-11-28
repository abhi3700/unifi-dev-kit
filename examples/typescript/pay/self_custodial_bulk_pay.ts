import { argv } from "node:process";
import { parsePaymentAmount } from "unifi-sdk-ts/primitives";
import type { StableCoin } from "unifi-sdk-ts/primitives";

import { client } from "../shared.ts";
import { expectedPermissions, needsApproval, paymentItem, readAddress, signPermit, testChain, testSigner } from "./permit2.ts";

const args = argv.slice(2);
const submit = args.at(-1) === "--submit";
if (submit) args.pop();
if (args.length !== 8) {
  throw new Error("Usage: self_custodial_bulk_pay.ts PAYER CHAIN RECIPIENT_1 AMOUNT_1 COIN_1 RECIPIENT_2 AMOUNT_2 COIN_2 [--submit]");
}
const [from, chainName, to1, amount1, coin1, to2, amount2, coin2] = args;
const payer = readAddress(from, "the payer");
const chain = testChain(chainName);
const batch = [
  paymentItem(payer, to1, amount1, coin1, "BulkPay"),
  paymentItem(payer, to2, amount2, coin2, "BulkPay"),
];
for (const item of batch) console.log(`Pay ${item.amount} ${item.coin} to ${item.to_address} on ${chain}.`);

const sdk = client();
const quote = await sdk.getSelfCustodialBulkPaymentPreflight(payer, false, chain, batch);
const feeCoin = batch[1].coin;
console.log(`Estimated batch fee ${quote.tot_est_fee} ${feeCoin}.`);
// The last item's coin pays the fee. Repeated coins share one total and allowance.
const amounts = [...batch, { coin: feeCoin, amount: quote.tot_est_fee }];
const totals = new Map<StableCoin, bigint>();
for (const { coin, amount } of amounts) {
  totals.set(coin, (totals.get(coin) ?? 0n) + parsePaymentAmount(amount, coin));
}
let needsTokenApproval = false;
for (const [coin, total] of totals) {
  const entry = quote.coin_entries[coin];
  if (!entry) throw new Error(`Missing ${coin} in batch quote`);
  console.log(`Safe to spend ${entry.balance} ${coin}.`);
  if (total > parsePaymentAmount(entry.balance, coin)) throw new Error(`Insufficient ${coin} balance for batch`);
  if (needsApproval(chain, coin, entry.allowance, total)) needsTokenApproval = true;
}
if (needsTokenApproval) {
  console.log("Confirm the token approvals in the payer's wallet, then rerun to refresh preflight.");
} else {
  console.log("Existing Permit2 allowances cover this batch.");
  if (submit) {
    const signer = testSigner(payer);
    const expected = expectedPermissions(amounts);
    const base = { chain, batch, tot_est_fee: quote.tot_est_fee, deadline: Math.floor(Date.now() / 1000) + 5 * 60 };
    // Keep payment order, fee, and deadline unchanged after requesting the permit.
    const permit = await sdk.createSelfCustodialBulkPaymentPermit(payer, base);
    const proof = await signPermit(signer, permit, expected, base.deadline);
    const result = await sdk.createSelfCustodialBulkPayment(payer, { payload: { base, ...proof }, save_batch: false })
      .catch((cause) => {
        throw new Error("Submission failed or status is unknown. Check payment history before trying again", { cause });
      });
    console.log({ bulkPayId: result.bulk_pay_id });
    batch.forEach((item, i) => console.log(`${item.to_address} → ${result.receipt_ids[i]}`));
  }
}
