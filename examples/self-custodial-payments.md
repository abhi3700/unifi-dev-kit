# Self-custodial payments with Permit2

Use a funded test wallet on Sepolia or UniFi's Anvil deployment. Set `API_KEY` for
the target API and use the wallet address as `PAYER`. The [Rust](rust/README.md) and
[TypeScript](typescript/README.md) examples take recipient addresses, decimal
amounts, and stablecoin symbols. Both add the estimated fee to the payment amounts.

## From preflight to receipt

1. **Preflight once for the current draft.** The response includes the spendable
   balance, allowance status, and estimated fee. A batch checks all selected coins
   in one call. Add the fee to the spend for the last item's coin. With two USDC
   payments of `1` and `2` and a fee of `0.05` USDC, the batch needs `3.05` USDC.
2. **Reuse sufficient allowances.** For each coin that needs approval, have the
   payer send one ERC-20 `approve(Permit2, totalSpendInBaseUnits)` transaction to the
   token contract. The `required_allowance` in preflight is a shortfall. ERC-20
   `approve` sets the full allowance, so use the displayed total including any fee
   charged in that coin. The wallet pays native gas for this transaction. Wait for
   the approval receipt and rerun preflight before signing.
3. **Request one permit when ready to sign.** Set a short expiry in Unix seconds.
   The permit endpoint supplies the nonce and EIP-712 message. The examples check
   the network, Permit2 contract, BundlePay spender, deadline, and ordered token
   amounts against the reviewed draft. The payer signs locally. The signature
   authorizes BundlePay to spend the specified amounts. Recipient addresses are
   supplied in the payment request outside the Permit2 message, so keep the
   reviewed request intact through submission.
4. **Submit once, then track the receipt.** Send the original base payload with
   the returned nonce as a decimal string and the signature. Keep the deadline,
   fee, and batch order fixed. A signing retry for the same draft can reuse its
   unexpired permit. If the draft changes, request and sign a new permit. If
   submission times out, check payment history before attempting another payment.
   A receipt ID means the API accepted the request. Read its status to track
   execution. Bulk submission returns a batch ID and one receipt ID per payment.

| Step | Rust SDK | TypeScript SDK |
| --- | --- | --- |
| Single preflight | `fetch_pre_ocp_balance_and_est_fee_ncw` | `getSelfCustodialPaymentPreflight` |
| Single permit | `create_domain_permit_for_sig` | `createSelfCustodialPaymentPermit` |
| Single submit | `pay_onchain_ncw` | `createSelfCustodialPayment` |
| Batch preflight | `fetch_pre_ocp_balance_and_est_fee_ncw_bulkpay` | `getSelfCustodialBulkPaymentPreflight` |
| Batch permit | `create_domain_permit_for_sig_bulkpay` | `createSelfCustodialBulkPaymentPermit` |
| Batch submit | `bulk_pay_ncw` | `createSelfCustodialBulkPayment` |
| Read receipts | `get_ocp_receipt` / `get_bulk_pay_receipts` | `getPaymentReceipt` / `getBulkPaymentReceipts` |

With sufficient allowances, submission takes three UniFi API calls in either flow.
Permit requests consume API credits and use the wallet's nonce issuance quota.
Keep permit creation out of form edits and call it once per reviewed payment or
batch. For an interactive Rust form that recalculates fees as amounts change, use
`prefetch_ncw_balance_fee_params` or its bulk counterpart with
`compute_est_fee_ncw` or `compute_est_fee_ncw_bulkpay` from the Rust primitives.

## Run with a test wallet

The examples show preflight by default. For `--submit`, set
`TEST_WALLET_PRIVATE_KEY` in your local environment using a disposable test wallet
that matches `PAYER`. Set `BUNDLEPAY_ADDRESS` and `USDC_ADDRESS`, `USDT_ADDRESS`,
or `DAI_ADDRESS` for the coins in your payment. Use the contract addresses from the
deployment behind `API_BASE_URL`. Hosted Sepolia and local test deployments may use
different token and BundlePay addresses.

UniFi Echo configures the following Sepolia addresses by mode.

| Environment variable | Hosted API | Local test stack |
| --- | --- | --- |
| `BUNDLEPAY_ADDRESS` | `0x58B096237Fb25E12EDc98D67Fc7B106589989fEc` | `0x773ad6250615144cc99Add429809C7B5c771b929` |
| `USDC_ADDRESS` | `0x4e53ccb52c7628226115b39680434309ad2c6af0` | `0xd07079aa4277a049c2fc24cb597768ba2729ea61` |
| `USDT_ADDRESS` | `0xa1c69afc7d82124addde6b4a2da1b9a169721273` | `0x3756f765a42882b7fe7e226176a63c584a8300ca` |
| `DAI_ADDRESS` | `0x28c5f011ad1f1445089cd9df962143de88237d89` | `0xea097538e759d11d89289f304c4bf2ee2b2a42cd` |

If the example asks for approval, send it from the payer's wallet. Set
`TOKEN_ADDRESS` to the token contract for the coin and use the Permit2 address and
total shown in the example output. With Foundry, run

```sh
cast send "$TOKEN_ADDRESS" 'approve(address,uint256)' "$PERMIT2_ADDRESS" "$TOTAL_BASE_UNITS" \
  --rpc-url "$RPC_URL" --interactive
```

Use the RPC for the chain served by the API. Once the approval succeeds, rerun the
example with `--submit` to sign and submit. In a browser integration, use the
payer's wallet to send the approval and sign the EIP-712 message, and keep the
UniFi API key on your backend.
