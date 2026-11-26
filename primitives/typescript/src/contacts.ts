import type { ChainProtocol } from "./types.js";

export interface ContactInput {
  name: string;
  protocol: ChainProtocol;
  address: string;
  is_own_wallet: boolean;
}

export interface Contact extends ContactInput {
  id: string;
  batch_ids: string[];
}

export interface ContactBatchListQuery {
  cursor?: string;
  limit?: number;
}

export interface ContactListQuery extends ContactBatchListQuery {
  with_batch?: boolean;
  protocol?: ChainProtocol;
  is_own_wallet?: boolean;
  search?: string;
  batch_cursor?: string;
  batch_limit?: number;
}

export interface ContactBatch {
  id: string;
  name?: string;
  contact_ids: string[];
  created_at_us: number;
}

export interface ContactPage {
  contacts: Contact[];
  total: number;
  next_cursor?: string;
  batches?: { batches: ContactBatch[]; total: number; next_cursor?: string };
}

export interface ContactBatchViewPage {
  batches: (Omit<ContactBatch, "contact_ids"> & { contacts: Contact[] })[];
  total: number;
  next_cursor?: string;
}

export interface UpsertContactsRequest {
  contacts?: ContactInput[];
  reject_existing?: boolean;
  updates?: { contact_id: string; update: Partial<ContactInput> }[];
  save_batch?: boolean;
  batch_id?: string;
  batch_contact_ids?: string[];
  batch_name?: string;
}

export interface UpsertContactsResult {
  contacts: Contact[];
  batch?: ContactBatch;
}

export interface ContactBatchDeletionImpact {
  id: string;
  name?: string;
  removed_contact_ids: string[];
  remaining_contact_ids: string[];
}

export interface DeleteContactsImpact {
  contact_ids: string[];
  unlinked_contact_ids: string[];
  batches_to_delete: ContactBatchDeletionImpact[];
  batches_to_delink: ContactBatchDeletionImpact[];
}
