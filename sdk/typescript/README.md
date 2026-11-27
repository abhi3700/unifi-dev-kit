# unifi-sdk-ts

TypeScript client for the UniFi v1 API. `UnifiClient` exposes wallet authentication,
contacts and contact batches, profile reads, sessions, wallet balances, and
managed and self-custodial payments. Payment methods cover preflight, single and bulk
submission, merchant checkout, Permit2, testnet faucet requests, receipts, and history.

Install from a commit with TypeScript support in the public developer kit.
Replace `COMMIT_SHA` with that commit's SHA.

```sh
npm install 'git+https://github.com/abhi3700/unifi-dev-kit.git#COMMIT_SHA'
```

```ts
import { UnifiClient } from "unifi-sdk-ts";

const unifi = new UnifiClient({ apiKey });
const balances = await unifi.getWalletBalances(userId);
```

Sign in to the [UniFi API page](https://payunifi.com/app/api) to create a key and keep it
on your server. The wallet challenge and proof exchange can use a
client constructed without a key. Set `baseUrl` for local or staging environments.
The client accepts the service root or a URL ending in `/v1`. Use strings for amounts
and balances to preserve decimal precision.

For quota alerts, read `(await unifi.getApiAccount(userId)).metadata.credits` from your
backend. Pass the user ID of the account that owns the API key. Each read costs one API
credit and returns the balance after the charge. Requests fail when the quota is exhausted.

Import shared API types and validators through `unifi-sdk-ts/primitives`.

```ts
import { isUniFiUserId, parsePaymentAmount } from "unifi-sdk-ts/primitives";
import type { PaymentReceipt } from "unifi-sdk-ts/primitives";
```

For self-custodial payments, `toPermit2TypedData` converts the permit response to viem's
`signTypedData` input. The wallet signs it and the application submits the signature
with the decimal nonce using a self-custodial payment method.

The [wallet and payment examples](../../examples/typescript/README.md) run from the
developer kit root with Node 22.18 or later.

`npm pack` from the developer kit root builds the SDK and packages both exports for local installation.
