import type { UserProfile } from "../../../primitives/typescript/src/account.js";

import { pathSegment } from "./transport.js";
import type { Transport } from "./transport.js";

export function profileMethods(transport: Transport) {
  return {
    getUserProfile(userId: string): Promise<UserProfile> {
      return transport.request(`/profile/${pathSegment(userId)}`);
    },
  };
}
