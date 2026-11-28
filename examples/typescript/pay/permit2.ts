import { env } from "node:process";
import { getAddress, isAddress, zeroAddress } from "viem";
import type { Address } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { formatPaymentAmount, parsePaymentAmount, toPermit2TypedData } from "unifi-sdk-ts/primitives";
import type {
  BulkPaymentItem, DomainPermitForSignature, PaymentAllowance, PaymentMemo, StableCoin,
} from "unifi-sdk-ts/primitives";

import { readChain, readCoin, required } from "../shared.ts";

export const permit2Address = getAddress("0x000000000022d473030f116ddee9f6b43ac78ba3");

export function testChain(value: string): "Sepolia" | "Anvil" {
  const chain = readChain(value);
  if (chain !== "Sepolia" && chain !== "Anvil") throw new Error("Use Sepolia or Anvil");
  return chain;
}

export function readAddress(value: string | undefined, name: string): Address {
  if (!value || !isAddress(value, { strict: false }) || value === zeroAddress) {
    throw new Error(`Provide a nonzero EVM address for ${name}`);
  }
  return getAddress(value);
}

export function paymentItem(
  payer: Address, recipient: string, amount: string, coinName: string, memo: PaymentMemo,
): BulkPaymentItem {
  const toAddress = readAddress(recipient, "the recipient");
  if (payer === toAddress) throw new Error("Choose a recipient other than the payer");
  const coin = readCoin(coinName);
  const units = parsePaymentAmount(amount, coin);
  if (units === 0n) throw new Error("Payment amount must be positive");
  return { coin, to_address: toAddress, amount: formatPaymentAmount(units, coin), memo };
}

export function needsApproval(
  chain: string, coin: StableCoin, allowance: PaymentAllowance, total: bigint,
): boolean {
  if (allowance.is_suff || allowance.is_max_allowance) return false;
  // ERC-20 approve sets the entire allowance. Use the total spend in base units.
  console.log(`Approve ${total} ${coin} base units on ${chain} to ${permit2Address}.`);
  return true;
}

export function testSigner(payer: Address) {
  const value = env.TEST_WALLET_PRIVATE_KEY?.replace(/^0x/, "");
  if (!value || !/^[0-9a-f]{64}$/i.test(value)) {
    throw new Error("Set TEST_WALLET_PRIVATE_KEY to a valid 32-byte wallet key");
  }
  const signer = privateKeyToAccount(`0x${value}`);
  if (signer.address !== payer) throw new Error("Test wallet key does not match the payer");
  return signer;
}

export function expectedPermissions(amounts: readonly { coin: StableCoin; amount: string }[]) {
  return {
    spender: readAddress(env.BUNDLEPAY_ADDRESS, "BUNDLEPAY_ADDRESS"),
    permitted: amounts.map(({ coin, amount }) => ({
      token: readAddress(env[`${coin}_ADDRESS`], `${coin}_ADDRESS`),
      amount: parsePaymentAmount(amount, coin),
    })),
  };
}

export async function signPermit(
  signer: ReturnType<typeof testSigner>,
  permit: DomainPermitForSignature,
  expected: ReturnType<typeof expectedPermissions>,
  deadline: number,
) {
  const typedData = toPermit2TypedData(permit);
  const { domain, message } = typedData;
  // UniFi's Anvil deployment uses the Sepolia chain ID.
  if (domain.name !== "Permit2" || domain.chainId !== 11155111n ||
      getAddress(domain.verifyingContract) !== permit2Address) {
    throw new Error("Unexpected Permit2 domain");
  }
  if (getAddress(message.spender) !== expected.spender) throw new Error("Unexpected BundlePay spender");
  if (message.deadline !== BigInt(deadline)) throw new Error("Unexpected Permit2 deadline");
  if (message.permitted.length !== expected.permitted.length ||
      message.permitted.some(({ token, amount }, i) =>
        getAddress(token) !== expected.permitted[i].token || amount !== expected.permitted[i].amount)) {
    throw new Error("Permit2 token permissions differ from the payment and fee");
  }

  const sig = await signer.signTypedData(typedData);
  return { nonce: message.nonce.toString(10), sig };
}
