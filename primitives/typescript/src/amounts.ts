import { maxUint256 } from "viem";
import { formatUnits, parseUnits } from "viem/utils";

import type { StableCoin } from "./types.js";

export const STABLE_COIN_DECIMALS = {
  USDT: 6,
  USDC: 6,
  DAI: 18,
} as const satisfies Record<StableCoin, number>;

/** Parses a human-readable decimal to base units without rounding. */
export function parsePaymentAmount(value: string, coin: StableCoin): bigint {
  const amount = value.trim();
  const decimals = STABLE_COIN_DECIMALS[coin];
  if (!/^(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)$/.test(amount)) {
    throw new RangeError("Invalid payment amount");
  }
  if ((amount.split(".")[1]?.length ?? 0) > decimals) {
    throw new RangeError(`Payment amount exceeds ${decimals} decimal places for ${coin}`);
  }
  const units = parseUnits(amount, decimals);
  if (units > maxUint256) throw new RangeError("Payment amount is too large");
  return units;
}

export function formatPaymentAmount(units: bigint, coin: StableCoin): string {
  return formatUnits(units, STABLE_COIN_DECIMALS[coin]);
}
