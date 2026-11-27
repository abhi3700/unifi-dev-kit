import type { ApiAccount } from "../../../primitives/typescript/src/account.js";

import { pathSegment } from "./transport.js";
import type { Transport } from "./transport.js";

export function apiMethods(transport: Transport) {
  return {
    /** Reads the key owner's account and returns the remaining balance after charging one API credit. */
    getApiAccount(userId: string): Promise<ApiAccount> {
      return transport.request(`/api/${pathSegment(userId)}`);
    },
  };
}
