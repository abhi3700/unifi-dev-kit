import type { FalconHeartbeat, UserSession } from "../../../primitives/typescript/src/account.js";

import { pathSegment } from "./transport.js";
import type { Transport } from "./transport.js";

export function sessionMethods(transport: Transport) {
  return {
    createSession(userId: string, heartbeat?: FalconHeartbeat): Promise<string> {
      return transport.request(`/session/create/${pathSegment(userId)}`, {
        method: "POST",
        body: { falcon_heartbeat: heartbeat },
      });
    },
    getSession(sessionId: string): Promise<UserSession> {
      return transport.request(`/session/${pathSegment(sessionId)}`);
    },
  };
}
