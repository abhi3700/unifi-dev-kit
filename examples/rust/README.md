# Rust examples

Run these commands from the developer kit root. Set `API_BASE_URL` and `API_KEY` in
your environment or local `.env`. See [`.env.template`](../../.env.template) for the
hosted API URL.

## Wallet

```sh
cargo run -p unifi-examples --example wallet_address
cargo run -p unifi-examples --example wallet_addresses
cargo run -p unifi-examples --example wallet_balances
```

## Payments

```sh
cargo run -p unifi-examples --example fetch_bal_fee
cargo run -p unifi-examples --example faucet_request
cargo run -p unifi-examples --example pay_receipt
cargo run -p unifi-examples --example pay_history
```

The existing [managed single payment](examples/pay/pay.rs) and
[managed bulk payment](examples/pay/bulk_pay.rs) examples submit payments when run.
Review their amounts and recipient addresses first.

For self-custodial funds, the examples [preflight by default](../self-custodial-payments.md)
and submit with `--submit`.

```sh
cargo run -p unifi-examples --example self_custodial_pay -- "$PAYER" Sepolia "$RECIPIENT_1" 1 USDC
cargo run -p unifi-examples --example self_custodial_bulk_pay -- "$PAYER" Sepolia "$RECIPIENT_1" 1 USDC "$RECIPIENT_2" 2 DAI
```

## Miscellaneous

```sh
cargo run -p unifi-examples --example api_credits -- "$KEY_OWNER_USER_ID"
cargo run -p unifi-examples --example contacts -- "$USER_ID"
cargo run -p unifi-examples --example contacts -- "$USER_ID" "$NEXT_CURSOR"
```

The credit check costs one API credit and reports the balance after that charge.
The contacts example reads one filtered page and prints the cursor for the next page.
