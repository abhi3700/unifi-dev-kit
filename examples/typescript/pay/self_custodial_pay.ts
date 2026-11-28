import { argv } from "node:process";
import { parsePaymentAmount } from "unifi-sdk-ts/primitives";

import { client } from "../shared.ts";
import { expectedPermissions, needsApproval, paymentItem, readAddress, signPermit, testChain, testSigner } from "./permit2.ts";

const args = argv.slice(2);
const submit = args.at(-1) === "--submit";
if (submit) args.pop();
if (args.length !== 5) {
  throw new Error("Usage: self_custodial_pay.ts PAYER CHAIN RECIPIENT AMOUNT COIN [--submit]");
}
const [from, chainName, recipient, amount, coinName] = args;
const payer = readAddress(from, "the payer");
const chain = testChain(chainName);
const payment = { chain, ...paymentItem(payer, recipient, amount, coinName, "General") };
const { coin } = payment;

const sdk = client();
// The fee is added to the recipient amount. This one call also checks allowance.
const quote = await sdk.getSelfCustodialPaymentPreflight(payer, false, payment);
const total = parsePaymentAmount(payment.amount, coin) + parsePaymentAmount(quote.est_fee, coin);
console.log(`Pay ${payment.amount} ${coin} to ${payment.to_address} on ${chain}. Estimated fee ${quote.est_fee} ${coin}. Safe to spend ${quote.balance} ${coin}.`);
if (total > parsePaymentAmount(quote.balance, coin)) throw new Error("Insufficient balance for payment and fee");
if (needsApproval(chain, coin, quote.allowance, total)) {
  console.log("Confirm the token approval in the payer's wallet, then rerun to refresh preflight.");
} else {
  console.log("Existing Permit2 allowance covers this payment.");
  if (submit) {
    const signer = testSigner(payer);
    const expected = expectedPermissions([payment, { coin, amount: quote.est_fee }]);
    const { memo, ...draft } = payment;
    const base = { ...draft, est_fee: quote.est_fee, deadline: Math.floor(Date.now() / 1000) + 5 * 60 };
    // Fetch a nonce only when ready to sign. Keep this base unchanged through submission.
    const permit = await sdk.createSelfCustodialPaymentPermit(payer, base);
    const proof = await signPermit(signer, permit, expected, base.deadline);
    const receiptId = await sdk.createSelfCustodialPayment(payer, { payload: { base, ...proof, memo } })
      .catch((cause) => {
        throw new Error("Submission failed or status is unknown. Check payment history before trying again", { cause });
      });
    console.log({ receiptId });
  }
}
