# TypeScript examples

Run these commands from `dev-kit/` with Node 22.18 or later. Run `npm ci` to install
dependencies and build the SDK. Set `API_KEY` in your environment or local `.env`.
Set `API_BASE_URL` to target a local API.

## Wallet

```sh
node --env-file-if-exists=.env examples/typescript/wallet/address.ts USER_ID Sepolia
node --env-file-if-exists=.env examples/typescript/wallet/addresses.ts USER_ID
node --env-file-if-exists=.env examples/typescript/wallet/balances.ts USER_ID
node --env-file-if-exists=.env examples/typescript/wallet/balances.ts USER_ID Sepolia USDC
```

The last command shows the latest and finalized balances for one pair.

## Payments

```sh
node --env-file-if-exists=.env examples/typescript/pay/faucet_request.ts USER_ID USDC
node --env-file-if-exists=.env examples/typescript/pay/fetch_bal_fee.ts USER_ID Sepolia USDC
node --env-file-if-exists=.env examples/typescript/pay/pay.ts USER_ID RECIPIENT 1 Sepolia USDC
node --env-file-if-exists=.env examples/typescript/pay/bulk_pay.ts USER_ID RECIPIENT_1 1 RECIPIENT_2 2 Sepolia USDC
node --env-file-if-exists=.env examples/typescript/pay/pay_receipt.ts RECEIPT_ID
node --env-file-if-exists=.env examples/typescript/pay/pay_history.ts USER_ID
```

The single and bulk payment examples show preflight by default. Append `--submit`
to send the payment. For the next history page, pass the last receipt ID as the
second argument to `pay_history.ts`.
