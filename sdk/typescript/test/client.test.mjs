import assert from "node:assert/strict";
import test from "node:test";

import { UnifiApiError, UnifiClient } from "unifi-sdk-ts";
import { isEvmAddress, isUniFiUserId, STABLE_COINS } from "unifi-sdk-ts/primitives";

function setup(reply, baseUrl = "http://127.0.0.1:3333/") {
  const calls = [];
  const client = new UnifiClient({
    apiKey: "test-key",
    baseUrl,
    fetch: async (url, init) => {
      calls.push(new Request(url, init));
      return reply();
    },
  });
  return { client, calls };
}

test("uses bearer auth, encodes path segments, and preserves decimal strings", async () => {
  const data = "0.000000000000000001";
  const { client, calls } = setup(() => Response.json({ status: "200 OK", data }));
  assert.equal(await client.getStablecoinBalance("alice/+?", "Anvil", "DAI"), data);
  assert.equal(calls[0].url, "http://127.0.0.1:3333/v1/wallet/onchain/balance/alice%2F%2B%3F/Anvil/DAI");
  assert.equal(calls[0].headers.get("authorization"), "Bearer test-key");
});

test("reuses address validation, including the Rust SDK's wallet address shortcut", async () => {
  const address = "0x2222222222222222222222222222222222222222";
  const { client, calls } = setup(() => assert.fail("Unexpected HTTP call"));
  assert.deepEqual(STABLE_COINS, ["USDT", "USDC", "DAI"]);
  assert.equal(isEvmAddress(address), true);
  assert.equal(isEvmAddress(address.slice(2)), true);
  assert.equal(isEvmAddress(`0x${"Aa".repeat(20)}`), true);
  assert.equal(isEvmAddress(address.slice(0, -1)), false);
  assert.equal(isEvmAddress(`0x${"g".repeat(40)}`), false);
  assert.equal(isUniFiUserId("A".repeat(24)), true);
  assert.equal(isUniFiUserId("g".repeat(24)), false);
  assert.equal(await client.getUserWalletAddress(address, "Anvil"), address);
  assert.deepEqual(await client.getUserWalletAddresses(address), { Evm: address });
  assert.deepEqual(calls, []);
});

test("reads account credits using the key owner's user ID", async () => {
  const data = {
    metadata: { plan: "Free", duration: "Month", credits: 0, img_credits: 10, expiry_at: 1900000000 },
    keys: [{ name: "quota-alerts", created_at: 1899000000 }],
  };
  const { client, calls } = setup(() => Response.json({ status: "200 OK", data }));
  assert.deepEqual(await client.getApiAccount("owner+alerts@example.com"), data);
  assert.equal(new URL(calls[0].url).pathname, "/v1/api/owner%2Balerts%40example.com");
  assert.equal(calls[0].method, "GET");
  assert.equal(calls[0].headers.get("authorization"), "Bearer test-key");
});

test("posts the managed payment wire format and returns a receipt ID", async () => {
  const request = {
    payload: {
      chain: "Sepolia",
      coin: "USDC",
      to_address: "0x2222222222222222222222222222222222222222",
      amount: "10.124",
      memo: { Custom: "Invoice 12" },
    },
  };
  const data = "receipt-id";
  const { client, calls } = setup(
    () => Response.json({ status: "200 OK", data }),
    "https://example.test/proxy/v1/",
  );
  assert.equal(await client.createManagedWalletPayment("alice", false, request), data);
  assert.equal(calls[0].url, "https://example.test/proxy/v1/payment/onchain/alice/false");
  assert.equal(calls[0].method, "POST");
  assert.equal(calls[0].headers.get("Content-Type"), "application/json");
  assert.deepEqual(await calls[0].json(), request);
});

test("encodes receipt history continuations and preserves false query values", async () => {
  const data = { has_prev: true, receipts: [], has_next: false };
  const { client, calls } = setup(() => Response.json({ status: "200 OK", data }));
  assert.deepEqual(
    await client.getPaymentHistory("alice", true, false, { pending: false, cursor: "a+b" }),
    data,
  );
  const url = new URL(calls[0].url);
  assert.equal(url.pathname, "/v1/payment/onchain/receipts/alice/true/false");
  assert.deepEqual([...url.searchParams], [["pending", "false"], ["cursor", "a+b"]]);
});

test("preserves the HTTP status and body and rejects malformed success envelopes", async () => {
  const { client } = setup(() => new Response("Unauthorized", { status: 401 }));
  await assert.rejects(client.healthCheck(), (error) => {
    assert.ok(error instanceof UnifiApiError);
    assert.equal(error.status, 401);
    assert.equal(error.body, "Unauthorized");
    return true;
  });
  const malformed = setup(() => Response.json({ status: "200 OK" })).client;
  await assert.rejects(malformed.healthCheck(), /Invalid UniFi API response/);
});
