import type {
  WalletAuthChallenge, WalletAuthChallengeRequest, WalletAuthProof,
} from "../../../primitives/typescript/src/account.js";

import { pathSegment } from "./transport.js";
import type { Transport } from "./transport.js";

export function authMethods(transport: Transport) {
  return {
    createWalletAuthChallenge(address: string, body: WalletAuthChallengeRequest): Promise<WalletAuthChallenge> {
      return transport.request(`/auth/challenge/${pathSegment(address)}`, { method: "POST", body, auth: false });
    },
    exchangeWalletAuthProof(address: string, body: WalletAuthProof): Promise<string> {
      return transport.request(`/auth/generate_jwt/${pathSegment(address)}`, { method: "POST", body, auth: false });
    },
  };
}
