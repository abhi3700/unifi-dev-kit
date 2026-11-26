import type { ApiAccount } from "../../../primitives/typescript/src/account.js";

import { pathSegment } from "./transport.js";
import type { Transport } from "./transport.js";

export function apiMethods(transport: Transport) {
  return {
    getApiAccount(userId: string): Promise<ApiAccount> {
      return transport.request(`/api/${pathSegment(userId)}`);
    },
    /** Returns the API key value, which is only available at creation. */
    generateApiKey(userId: string, name: string): Promise<string> {
      return transport.request(`/api_key/generate/${pathSegment(userId)}/${pathSegment(name)}`, { method: "POST" });
    },
    deleteApiKey(userId: string, name: string): Promise<void> {
      return transport.requestVoid(`/api_key/del/${pathSegment(userId)}/${pathSegment(name)}`, { method: "DELETE" });
    },
  };
}
