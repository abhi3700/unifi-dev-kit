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

The managed single and bulk payment examples show preflight by default. Append
`--submit` to send the payment. For the next history page, pass the last receipt ID
as the second argument to `pay_history.ts`.

### Self-custodial payments

See the [Permit2 guide](../self-custodial-payments.md) for wallet setup, ERC-20
approval, and signing. The examples preflight by default and submit with `--submit`.

```sh
node --env-file-if-exists=.env examples/typescript/pay/self_custodial_pay.ts "$PAYER" Sepolia "$RECIPIENT_1" 1 USDC
node --env-file-if-exists=.env examples/typescript/pay/self_custodial_bulk_pay.ts "$PAYER" Sepolia "$RECIPIENT_1" 1 USDC "$RECIPIENT_2" 2 DAI
```

## Miscellaneous

```sh
node --env-file-if-exists=.env examples/typescript/misc/api_credits.ts "$KEY_OWNER_USER_ID"
node --env-file-if-exists=.env examples/typescript/misc/contacts.ts "$USER_ID"
node --env-file-if-exists=.env examples/typescript/misc/contacts.ts "$USER_ID" "$NEXT_CURSOR"
```

The credit check costs one API credit and reports the balance after that charge.
The contacts example reads one filtered page and prints the cursor for the next page.
