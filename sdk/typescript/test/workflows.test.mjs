import assert from "node:assert/strict";
import test from "node:test";

import { hashTypedData } from "viem";
import { UnifiClient } from "unifi-sdk-ts";
import { formatPaymentAmount, parsePaymentAmount, toPermit2TypedData } from "unifi-sdk-ts/primitives";

function setup(reply, apiKey = "test-key") {
  const calls = [];
  const client = new UnifiClient({
    apiKey: apiKey ?? undefined,
    baseUrl: "http://127.0.0.1:3333/",
    fetch: async (url, init) => {
      const request = new Request(url, init);
      calls.push(request);
      return reply(request);
    },
  });
  return { client, calls };
}

function data(value) {
  return Response.json({ status: "200 OK", data: value });
}

test("bulk payment methods send canonical preflight coins and preserve recipient order", async () => {
  const { client, calls } = setup((request) => (
    request.url.includes("balance_est_fee_bulkpay")
      ? data({ coin_entries: [], gas_price: 1, gas_token_price: 1, contacts_total: 0 })
      : data({ bulk_pay_id: "batch-id", receipt_ids: ["r1", "r2"] })
  ));
  await client.getManagedBulkPaymentPreflight("alice", "Sepolia", ["DAI", "USDC", "DAI"], true);
  assert.equal(calls[0].url, "http://127.0.0.1:3333/v1/payment/onchain/balance_est_fee_bulkpay/alice/Sepolia?fetch_contacts=true");
  assert.deepEqual(await calls[0].json(), { coins: ["USDC", "DAI"] });
  assert.throws(() => client.getManagedBulkPaymentPreflight("alice", "Sepolia", []), /at least one/);
  assert.throws(
    () => client.getManagedBulkPaymentPreflight("alice", "Sepolia", ["DAI", "ETH"]),
    /Unsupported stablecoin/,
  );

  const batch = ["USDC", "DAI"].map((coin, index) => ({
    coin, to_address: `0x${String(index + 1).repeat(40)}`, amount: "1.23", memo: "BulkPay",
  }));
  const result = await client.createManagedBulkPayment("alice", false, {
    chain: "Sepolia", batch, save_batch: false,
  });
  assert.equal(calls[1].url, "http://127.0.0.1:3333/v1/payment/onchain/cw_bulkpay/alice/false");
  assert.deepEqual((await calls[1].json()).batch, batch);
  assert.deepEqual(result.receipt_ids, ["r1", "r2"]);
});

test("wallet auth requests omit bearer auth and can start without an API key", async () => {
  const address = "0x1111111111111111111111111111111111111111";
  const challenge = { message: "Sign in", nonce: "test-nonce", expires_at: 100 };
  const { client, calls } = setup(
    (request) => data(request.url.includes("/challenge/") ? challenge : "test-jwt"),
    null,
  );
  const challengeRequest = { domain: "example.test", uri: "https://example.test", chain_id: 1 };
  assert.deepEqual(await client.createWalletAuthChallenge(address, challengeRequest), challenge);
  assert.deepEqual(await calls[0].json(), challengeRequest);
  assert.equal(new URL(calls[0].url).pathname, `/v1/auth/challenge/${address}`);
  assert.equal(calls[0].headers.get("authorization"), null);

  const proof = { message: challenge.message, nonce: challenge.nonce, signature: "0x11", platform: "Web" };
  assert.equal(await client.exchangeWalletAuthProof(address, proof), "test-jwt");
  assert.deepEqual(await calls[1].json(), proof);
  assert.equal(new URL(calls[1].url).pathname, `/v1/auth/generate_jwt/${address}`);
  const keyed = setup(() => data("test-jwt"));
  await keyed.client.exchangeWalletAuthProof(address, proof);
  assert.equal(keyed.calls[0].headers.get("authorization"), null);
  await assert.rejects(client.getWalletBalances("alice"), /API key is required/);
  assert.equal(calls.length, 2);
});

test("contact and account mutations preserve DELETE bodies and handle responses without data", async () => {
  const { client, calls } = setup(() => Response.json({ status: "200 OK", message: "Done" }));
  await client.deleteContactBatch("alice", "a/b", false);
  await client.setUserProfile("alice", { nickname: "Al" });
  await client.deleteApiKey("alice", "my key");

  assert.equal(calls[0].method, "DELETE");
  assert.equal(new URL(calls[0].url).pathname, "/v1/contact_batches/alice/a%2Fb");
  assert.deepEqual(await calls[0].json(), { del_its_contacts: false });
  assert.equal(calls[1].method, "POST");
  assert.deepEqual(await calls[1].json(), { nickname: "Al" });
  assert.equal(calls[2].method, "DELETE");
  assert.equal(new URL(calls[2].url).pathname, "/v1/api_key/del/alice/my%20key");
});

test("single self-custodial payment uses separate Permit2 and submission wire bodies", async () => {
  const verifyingContract = "0x000000000022d473030f116ddee9f6b43ac78ba3";
  const token = "0x1111111111111111111111111111111111111111";
  const spender = "0x2222222222222222222222222222222222222222";
  const permit = {
    domain: { name: "Permit2", chainId: "0xaa36a7", verifyingContract },
    permit: {
      permitted: [{ token, amount: "0x20000000000001" }],
      spender, nonce: "0x2", deadline: "1893456000",
    },
  };
  const { client, calls } = setup((request) => (
    request.url.includes("create_domain_permit") ? data(permit) : data("receipt-id")
  ));
  const base = { chain: "Sepolia", coin: "DAI", to_address: spender, amount: "1", est_fee: "0.1", deadline: 1893456000 };
  const typedData = toPermit2TypedData(await client.createSelfCustodialPaymentPermit("alice", base));
  assert.deepEqual(await calls[0].json(), base);
  assert.equal(typedData.domain.chainId, 11155111n);
  assert.equal(typedData.message.permitted[0].amount, 9007199254740993n);
  assert.equal(typedData.message.nonce, 2n);
  assert.match(hashTypedData(typedData), /^0x[0-9a-f]{64}$/);

  const payload = { base, nonce: typedData.message.nonce.toString(), sig: "0x11", memo: "General" };
  assert.equal(await client.createSelfCustodialPayment("alice", { payload }), "receipt-id");
  assert.equal(new URL(calls[1].url).pathname, "/v1/payment/onchain/ncw/alice");
  assert.deepEqual(await calls[1].json(), { payload });
});

test("payment amounts preserve base units and reject excess precision instead of rounding", () => {
  assert.equal(parsePaymentAmount(" 1.234567 ", "USDC"), 1234567n);
  assert.equal(parsePaymentAmount(".000000000000000001", "DAI"), 1n);
  assert.equal(formatPaymentAmount(1n, "DAI"), "0.000000000000000001");
  assert.throws(() => parsePaymentAmount("0.1234567", "USDC"), /6 decimal places/);
  assert.throws(() => parsePaymentAmount("1e3", "DAI"), /Invalid payment amount/);
  assert.throws(() => parsePaymentAmount((2n ** 256n).toString(), "DAI"), /too large/);
});
