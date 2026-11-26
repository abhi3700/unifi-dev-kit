import type { ChainName, StableCoin } from "./types.js";

export interface WalletBalance {
  balance: string;
  value_usd: string;
}

export interface WalletBalancesByCoin {
  total_usd: string;
  price_usd: string;
  chain_details: [ChainName, WalletBalance][];
}

export type WalletBalances = [StableCoin, WalletBalancesByCoin][];

export interface WalletBalancesByChain {
  total_usd: string;
  coin_details: [StableCoin, WalletBalance & { price_usd: string }][];
}

export interface WalletBalanceByPair {
  balance: string;
  latest_balance: string;
  latest_value_usd: string;
  finalized_balance: string;
  finalized_value_usd: string;
  finality_status: "synced" | "incoming_pending" | "outgoing_pending";
  finality_delta: string;
}
