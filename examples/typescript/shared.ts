import { env } from "node:process";

import { CHAIN_NAMES, STABLE_COINS, UnifiClient } from "unifi-sdk-ts";
import type { ChainName, StableCoin } from "unifi-sdk-ts";

export function client(): UnifiClient {
  if (!env.API_KEY) throw new Error("Set API_KEY before running the example");
  return new UnifiClient({ apiKey: env.API_KEY, baseUrl: env.API_BASE_URL });
}

export function required(value: string | undefined, name: string): string {
  if (!value) throw new Error(`Pass ${name} as a command-line argument`);
  return value;
}

export function readChain(value = "Sepolia"): ChainName {
  const chain = CHAIN_NAMES.find((name) => name === value);
  if (!chain) throw new Error(`Choose a chain from ${CHAIN_NAMES.join(", ")}`);
  return chain;
}

export function readCoin(value = "USDC"): StableCoin {
  const coin = STABLE_COINS.find((name) => name === value);
  if (!coin) throw new Error(`Choose a stablecoin from ${STABLE_COINS.join(", ")}`);
  return coin;
}
