export const CHAIN_NAMES = ["Ethereum", "Polygon", "Sepolia", "Anvil"] as const;
export const STABLE_COINS = ["USDT", "USDC", "DAI"] as const;

export type ChainName = (typeof CHAIN_NAMES)[number];
export type StableCoin = (typeof STABLE_COINS)[number];
export type ChainProtocol = "Evm";

export interface ApiResponse<T> {
  status: string;
  data: T;
}

export function dedupeStableCoins(coins: readonly StableCoin[]): StableCoin[] {
  const selected = new Set(coins);
  const canonical = STABLE_COINS.filter((coin) => selected.has(coin));
  if (canonical.length !== selected.size) throw new RangeError("Unsupported stablecoin");
  return canonical;
}
