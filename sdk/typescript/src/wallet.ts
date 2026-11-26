import { isEvmAddress } from "../../../primitives/typescript/src/evm.js";
import type {
  ChainName, ChainProtocol, StableCoin, WalletBalance, WalletBalanceByPair,
  WalletBalances, WalletBalancesByChain, WalletBalancesByCoin,
} from "../../../primitives/typescript/src/index.js";

import { pathSegment } from "./transport.js";
import type { Transport } from "./transport.js";

export function walletMethods(transport: Transport) {
  async function getUserWalletAddress(userId: string, chain: ChainName): Promise<string> {
    if (isEvmAddress(userId)) {
      return userId;
    }
    return transport.request(`/wallet/address/${pathSegment(userId)}/${pathSegment(chain)}`);
  }

  async function getUserWalletAddresses(userId: string): Promise<Partial<Record<ChainProtocol, string>>> {
    if (isEvmAddress(userId)) {
      return { Evm: userId };
    }
    return transport.request(`/wallet/addresses/${pathSegment(userId)}`);
  }

  function getStablecoinBalance(userId: string, chain: ChainName, coin: StableCoin): Promise<string> {
    return transport.request(
      `/wallet/onchain/balance/${pathSegment(userId)}/${pathSegment(chain)}/${pathSegment(coin)}`,
    );
  }

  function getChainStablecoinBalances(userId: string, chain: ChainName): Promise<Partial<Record<StableCoin, string>>> {
    return transport.request(`/wallet/onchain/balances/${pathSegment(userId)}/${pathSegment(chain)}`);
  }

  function getWalletBalances(userId: string): Promise<WalletBalances> {
    return transport.request(`/wallet/balances/${pathSegment(userId)}`);
  }

  function getWalletBalancesByChain(userId: string, chain: ChainName): Promise<WalletBalancesByChain> {
    return transport.request(`/wallet/balances/by_chain/${pathSegment(userId)}/${pathSegment(chain)}`);
  }

  function getWalletBalancesByCoin(userId: string, coin: StableCoin): Promise<WalletBalancesByCoin> {
    return transport.request(`/wallet/balances/by_coin/${pathSegment(userId)}/${pathSegment(coin)}`);
  }

  async function getWalletBalanceByPair(userId: string, chain: ChainName, coin: StableCoin): Promise<WalletBalance> {
    const { balance, latest_value_usd } = await getWalletBalanceByPairWithFinality(userId, chain, coin);
    return { balance, value_usd: latest_value_usd };
  }

  function getWalletBalanceByPairWithFinality(userId: string, chain: ChainName, coin: StableCoin): Promise<WalletBalanceByPair> {
    return transport.request(
      `/wallet/balance/by_pair/${pathSegment(userId)}/${pathSegment(chain)}/${pathSegment(coin)}`,
    );
  }

  return {
    getUserWalletAddress,
    getUserWalletAddresses,
    getStablecoinBalance,
    getChainStablecoinBalances,
    getWalletBalances,
    getWalletBalancesByChain,
    getWalletBalancesByCoin,
    getWalletBalanceByPair,
    getWalletBalanceByPairWithFinality,
  };
}
