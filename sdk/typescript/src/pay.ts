import type {
  BulkPaymentItem,
  BulkPaymentPreflight,
  BulkPaymentReceiptGroup,
  BulkPaymentResult,
  ManagedBulkPaymentRequest,
  ManagedWalletPaymentRequest,
  PaymentHistory,
  PaymentHistoryQuery,
  PaymentPayload,
  PaymentPreflight,
  PaymentReceipt,
  SelfCustodialBulkPaymentBase,
  SelfCustodialBulkPaymentPreflight,
  SelfCustodialBulkPaymentRequest,
  SelfCustodialPaymentBase,
  SelfCustodialPaymentFeeParams,
  SelfCustodialPaymentPayload,
  SelfCustodialPaymentPreflight,
  SelfCustodialPaymentRequest,
} from "../../../primitives/typescript/src/payment.js";
import type { DomainPermitForSignature } from "../../../primitives/typescript/src/permit2.js";
import { dedupeStableCoins } from "../../../primitives/typescript/src/types.js";
import type { ChainName, StableCoin } from "../../../primitives/typescript/src/types.js";

import { pathSegment } from "./transport.js";
import type { Transport } from "./transport.js";

const onchain = "/payment/onchain";

function paymentPair(userId: string, chain: ChainName, coin: StableCoin): string {
  return `${pathSegment(userId)}/${pathSegment(chain)}/${pathSegment(coin)}`;
}

function bulkPreflightBody(coins: readonly StableCoin[]) {
  const selected = dedupeStableCoins(coins);
  if (!selected.length) throw new RangeError("Select at least one stablecoin");
  return { coins: selected };
}

export function paymentMethods(transport: Transport) {
  return {
    getPaymentNetBalance(userId: string, chain: ChainName, coin: StableCoin): Promise<string> {
      return transport.request(`${onchain}/net_balance/${paymentPair(userId, chain, coin)}`);
    },
    getEstimatedPaymentFee(userId: string, chain: ChainName, coin: StableCoin): Promise<string> {
      return transport.request(`${onchain}/est_fee/${paymentPair(userId, chain, coin)}`);
    },
    getPaymentPreflight(
      userId: string, chain: ChainName, coin: StableCoin, fetchContacts = false,
    ): Promise<PaymentPreflight> {
      return transport.request(`${onchain}/balance_est_fee/${paymentPair(userId, chain, coin)}`, {
        query: { fetch_contacts: fetchContacts },
      });
    },
    getManagedBulkPaymentPreflight(
      userId: string, chain: ChainName, coins: readonly StableCoin[], fetchContacts = false,
    ): Promise<BulkPaymentPreflight> {
      return transport.request(`${onchain}/balance_est_fee_bulkpay/${pathSegment(userId)}/${pathSegment(chain)}`, {
        method: "POST",
        body: bulkPreflightBody(coins),
        query: { fetch_contacts: fetchContacts },
      });
    },
    getSelfCustodialPaymentPreflight(
      userId: string, isFeeInclusive: boolean, body: PaymentPayload,
    ): Promise<SelfCustodialPaymentPreflight> {
      return transport.request(`${onchain}/balance_est_fee_ncw/${pathSegment(userId)}/${isFeeInclusive}`, {
        method: "POST", body,
      });
    },
    getSelfCustodialBulkPaymentPreflight(
      userId: string, isFeeInclusive: boolean, chain: ChainName, items: readonly BulkPaymentItem[],
    ): Promise<SelfCustodialBulkPaymentPreflight> {
      return transport.request(`${onchain}/balance_est_fee_ncw_bulkpay/${pathSegment(userId)}/${isFeeInclusive}`, {
        method: "POST", body: { chain, items },
      });
    },
    prefetchSelfCustodialPaymentFeeParams(
      userId: string, chain: ChainName, coin: StableCoin, fetchContacts = false,
    ): Promise<SelfCustodialPaymentFeeParams> {
      return transport.request(`${onchain}/ncw/prefetch_balance_est_fee_params/${paymentPair(userId, chain, coin)}`, {
        query: { fetch_contacts: fetchContacts },
      });
    },
    prefetchSelfCustodialBulkPaymentFeeParams(
      userId: string, chain: ChainName, coins: readonly StableCoin[], fetchContacts = false,
    ): Promise<BulkPaymentPreflight> {
      return transport.request(`${onchain}/ncw/prefetch_balance_est_fee_params_bulkpay/${pathSegment(userId)}/${pathSegment(chain)}`, {
        method: "POST",
        body: bulkPreflightBody(coins),
        query: { fetch_contacts: fetchContacts },
      });
    },
    /** Returns the resulting session ID when one was requested. */
    requestFaucet(userId: string, coin: StableCoin, chain: "Sepolia" | "Anvil", createSession = false): Promise<string> {
      const path = `/faucet/${pathSegment(userId)}/${pathSegment(coin)}/${pathSegment(chain)}/${createSession}`;
      return transport.request(path, { method: "POST" });
    },
    /** Returns the receipt ID for the submitted payment. */
    createManagedWalletPayment(userId: string, isFeeInclusive: boolean, body: ManagedWalletPaymentRequest): Promise<string> {
      return transport.request(`${onchain}/${pathSegment(userId)}/${isFeeInclusive}`, { method: "POST", body });
    },
    createManagedBulkPayment(
      userId: string, isFeeInclusive: boolean, body: ManagedBulkPaymentRequest,
    ): Promise<BulkPaymentResult> {
      return transport.request(`${onchain}/cw_bulkpay/${pathSegment(userId)}/${isFeeInclusive}`, { method: "POST", body });
    },
    createManagedMerchantPayment(
      userId: string, isFeeInclusive: boolean, sessionId: string, body: PaymentPayload,
    ): Promise<string> {
      return transport.request(`${onchain}/merchant/${pathSegment(userId)}/${isFeeInclusive}/${pathSegment(sessionId)}`, {
        method: "POST", body,
      });
    },
    createSelfCustodialPaymentPermit(
      userId: string, body: SelfCustodialPaymentBase,
    ): Promise<DomainPermitForSignature> {
      return transport.request(`${onchain}/ncw/create_domain_permit/${pathSegment(userId)}`, { method: "POST", body });
    },
    createSelfCustodialBulkPaymentPermit(
      userId: string, body: SelfCustodialBulkPaymentBase,
    ): Promise<DomainPermitForSignature> {
      return transport.request(`${onchain}/ncw/create_domain_permit_bulkpay/${pathSegment(userId)}`, { method: "POST", body });
    },
    createSelfCustodialPayment(userId: string, body: SelfCustodialPaymentRequest): Promise<string> {
      return transport.request(`${onchain}/ncw/${pathSegment(userId)}`, { method: "POST", body });
    },
    createSelfCustodialBulkPayment(userId: string, body: SelfCustodialBulkPaymentRequest): Promise<BulkPaymentResult> {
      return transport.request(`${onchain}/ncw_bulkpay/${pathSegment(userId)}`, { method: "POST", body });
    },
    createSelfCustodialMerchantPayment(
      userId: string, sessionId: string, body: SelfCustodialPaymentPayload,
    ): Promise<string> {
      return transport.request(`${onchain}/ncw/merchant/${pathSegment(userId)}/${pathSegment(sessionId)}`, {
        method: "POST", body,
      });
    },
    notifyFliqPayer(paymentId: string, payment: Omit<PaymentPayload, "memo">): Promise<void> {
      const parts = [paymentId, payment.chain, payment.coin, payment.to_address, payment.amount].map(pathSegment);
      return transport.requestVoid(`${onchain}/fliq/notify/payer/${parts.join("/")}`);
    },
    /** Returns an empty string while the merchant session has no payment receipt. */
    getPaymentSessionStatus(sessionId: string): Promise<string> {
      return transport.request(`/payment/merchant/session/${pathSegment(sessionId)}`);
    },
    getPaymentReceipt(receiptId: string): Promise<PaymentReceipt> {
      return transport.request(`${onchain}/receipt/${pathSegment(receiptId)}`);
    },
    getBulkPaymentReceipts(bulkPayId: string): Promise<BulkPaymentReceiptGroup> {
      return transport.request(`${onchain}/bulkpay/${pathSegment(bulkPayId)}`);
    },
    getPaymentHistory(
      userId: string, sortByLatest: boolean, fromStart: boolean, query: PaymentHistoryQuery = {},
    ): Promise<PaymentHistory> {
      return transport.request(`${onchain}/receipts/${pathSegment(userId)}/${sortByLatest}/${fromStart}`, { query });
    },
  };
}
