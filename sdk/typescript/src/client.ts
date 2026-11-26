import { apiMethods } from "./api.js";
import { authMethods } from "./auth.js";
import { contactMethods } from "./contacts.js";
import { paymentMethods } from "./pay.js";
import { profileMethods } from "./profile.js";
import { sessionMethods } from "./session.js";
import { createTransport } from "./transport.js";
import type { UnifiClientOptions } from "./transport.js";
import { walletMethods } from "./wallet.js";

export interface UnifiClient
  extends ReturnType<typeof apiMethods>,
    ReturnType<typeof authMethods>,
    ReturnType<typeof contactMethods>,
    ReturnType<typeof paymentMethods>,
    ReturnType<typeof profileMethods>,
    ReturnType<typeof sessionMethods>,
    ReturnType<typeof walletMethods> {}

export class UnifiClient {
  #transport: ReturnType<typeof createTransport>;

  constructor(options: UnifiClientOptions = {}) {
    const transport = createTransport(options);
    this.#transport = transport;
    // Keep the public methods on one client while sharing a single transport.
    Object.assign(
      this,
      apiMethods(transport),
      authMethods(transport),
      contactMethods(transport),
      paymentMethods(transport),
      profileMethods(transport),
      sessionMethods(transport),
      walletMethods(transport),
    );
  }

  healthCheck(): Promise<string> {
    return this.#transport.request("/health");
  }
}
