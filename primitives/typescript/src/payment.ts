import type { ApiPlan, PaidPlanDuration } from "./account.js";
import type { Contact } from "./contacts.js";
import type { ChainName, StableCoin } from "./types.js";

export const RECEIPT_STATUSES = [
  "Processing",
  "Failed",
  "Confirmed",
  "Finalized",
  "Reorged",
] as const;

export type ReceiptStatus = (typeof RECEIPT_STATUSES)[number];

export interface PaymentPreflight {
  is_coin_allowance_zero: boolean;
  balance: string;
  est_fee: string;
  contacts?: Contact[];
  contacts_total: number;
  contacts_next_cursor?: string;
}

export type PaymentMemo =
  | "General"
  | "FliqPay"
  | "FliqPayMerchant"
  | "BulkPay"
  | { Custom: string }
  | { SubscribeApi: [ApiPlan, PaidPlanDuration] };

export interface BulkPaymentItem {
  coin: StableCoin;
  to_address: string;
  /** Amount in human-readable token units, e.g. "10.124". */
  amount: string;
  memo: PaymentMemo;
}

export interface PaymentPayload extends BulkPaymentItem {
  chain: ChainName;
}

export interface PaymentSchedule {
  id: string;
  is_recurring: boolean;
}

export interface ManagedWalletPaymentRequest {
  payload: PaymentPayload;
  sched?: PaymentSchedule | null;
}

export interface PaymentReceipt {
  id: string;
  bulk_pay_id?: string;
  bundle_id?: number;
  entity: string;
  user_id: string;
  is_fee_incl: boolean;
  chain: ChainName;
  coin: StableCoin;
  to_address: string;
  amount: string;
  memo: PaymentMemo;
  fee_coin?: StableCoin;
  est_fee: string;
  act_fee: string;
  tx_hash: string;
  block_num: number;
  status: ReceiptStatus;
  start_ts_us: number;
  end_ts_us: number;
}

export interface PaymentHistoryQuery {
  chain?: ChainName;
  coin?: StableCoin;
  status?: ReceiptStatus;
  /** Mutually exclusive with status. */
  pending?: boolean;
  direction?: "Sent" | "Received";
  limit?: number;
  next_or_previous?: boolean;
  cursor?: string;
}

export interface PaymentHistory {
  has_prev: boolean;
  receipts: PaymentReceipt[];
  has_next: boolean;
}

interface SavePaymentBatch {
  scheds?: PaymentSchedule[] | null;
  save_batch: boolean;
  batch_name?: string;
}

export interface ManagedBulkPaymentRequest extends SavePaymentBatch {
  chain: ChainName;
  batch: BulkPaymentItem[];
}

export interface BulkPaymentResult {
  bulk_pay_id: string;
  receipt_ids: string[];
}

export interface BulkPaymentReceiptGroup {
  bulk_pay_id: string;
  receipts: PaymentReceipt[];
}

export interface BulkPaymentPreflight {
  coin_entries: {
    coin: StableCoin;
    balance: string;
    allowance: string;
    coin_price: number;
  }[];
  gas_price: number;
  gas_token_price: number;
  contacts?: Contact[];
  contacts_total: number;
  contacts_next_cursor?: string | null;
}

export interface PaymentAllowance {
  is_suff: boolean;
  required_allowance: string;
  is_max_allowance: boolean;
}

interface SelfCustodialBalance {
  allowance: PaymentAllowance;
  balance: string;
}

export interface SelfCustodialPaymentPreflight extends SelfCustodialBalance {
  est_fee: string;
}

export interface SelfCustodialBulkPaymentPreflight {
  coin_entries: Partial<Record<StableCoin, SelfCustodialBalance>>;
  tot_est_fee: string;
}

export interface SelfCustodialPaymentFeeParams {
  allowance: string;
  balance: string;
  gas_price: number;
  gas_token_price: number;
  coin_price: number;
  contacts?: Contact[];
  contacts_total: number;
  contacts_next_cursor?: string;
}

export interface SelfCustodialPaymentBase {
  chain: ChainName;
  coin: StableCoin;
  to_address: string;
  amount: string;
  est_fee: string;
  deadline: number;
}

export interface SelfCustodialBulkPaymentBase {
  chain: ChainName;
  batch: BulkPaymentItem[];
  tot_est_fee: string;
  deadline: number;
}

interface SignedPermit<T> {
  base: T;
  /** Decimal Permit2 nonce. */
  nonce: string;
  sig: string;
}

export interface SelfCustodialPaymentPayload extends SignedPermit<SelfCustodialPaymentBase> {
  memo: PaymentMemo;
}

export interface SelfCustodialPaymentRequest {
  payload: SelfCustodialPaymentPayload;
  sched?: PaymentSchedule | null;
}

export interface SelfCustodialBulkPaymentRequest extends SavePaymentBatch {
  payload: SignedPermit<SelfCustodialBulkPaymentBase>;
}
