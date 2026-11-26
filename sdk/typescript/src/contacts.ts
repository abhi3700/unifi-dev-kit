import type {
  Contact, ContactBatch, ContactBatchListQuery, ContactBatchViewPage, ContactListQuery,
  ContactPage, DeleteContactsImpact, UpsertContactsRequest, UpsertContactsResult,
} from "../../../primitives/typescript/src/contacts.js";

import { pathSegment } from "./transport.js";
import type { Transport } from "./transport.js";

export function contactMethods(transport: Transport) {
  return {
    getContacts(userId: string, query?: ContactListQuery): Promise<ContactPage> {
      return transport.request(`/contacts/${pathSegment(userId)}`, { query });
    },
    getContact(userId: string, contactId: string): Promise<Contact> {
      return transport.request(`/contacts/${pathSegment(userId)}/${pathSegment(contactId)}`);
    },
    upsertContacts(userId: string, body: UpsertContactsRequest): Promise<UpsertContactsResult> {
      return transport.request(`/contacts/${pathSegment(userId)}`, { method: "POST", body });
    },
    previewDeleteContacts(userId: string, contactIds: string[]): Promise<DeleteContactsImpact> {
      return transport.request(`/contacts/${pathSegment(userId)}/delete_impact`, {
        method: "POST", body: { contact_ids: contactIds },
      });
    },
    deleteContacts(userId: string, contactIds: string[]): Promise<DeleteContactsImpact> {
      return transport.request(`/contacts/${pathSegment(userId)}`, {
        method: "DELETE", body: { contact_ids: contactIds },
      });
    },
    getContactBatches(userId: string, query?: ContactBatchListQuery): Promise<ContactBatchViewPage> {
      return transport.request(`/contact_batches/${pathSegment(userId)}`, { query });
    },
    getContactBatch(userId: string, batchId: string): Promise<ContactBatch> {
      return transport.request(`/contact_batches/${pathSegment(userId)}/${pathSegment(batchId)}`);
    },
    deleteContactBatch(userId: string, batchId: string, deleteItsContacts: boolean): Promise<void> {
      return transport.requestVoid(`/contact_batches/${pathSegment(userId)}/${pathSegment(batchId)}`, {
        method: "DELETE", body: { del_its_contacts: deleteItsContacts },
      });
    },
  };
}
