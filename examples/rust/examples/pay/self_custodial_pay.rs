//! Preflight one payment, reuse its ERC-20 allowance, and sign one Permit2 message.

mod permit2;

use eyre::{Context, bail, ensure};
use unifi_examples::init_sdk;
use unifi_sdk_primitives::{
	types::{
		Memo, PayOnchainNcwRequest, PayOnchainPayload, PayOnchainPayloadNcw,
		PayOnchainPayloadNcwBase,
	},
	utils::{now_timestamp_secs, parse_payment_amount},
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let (args, submit) = permit2::args();
	let [payer, chain, recipient, amount, coin] = args.as_slice() else {
		bail!("Usage: self_custodial_pay PAYER CHAIN RECIPIENT AMOUNT COIN [--submit]");
	};
	let payer = permit2::payer_address(payer)?;
	let user_id = payer.to_string();
	let chain = permit2::test_chain(chain)?;
	let item = permit2::payment_item(payer, recipient, amount, coin, Memo::General)?;
	let payment = PayOnchainPayload {
		chain,
		coin: item.coin,
		to_address: item.to_address,
		amount: item.amount,
		memo: item.memo,
	};

	let sdk = init_sdk();
	// The fee is added to the recipient amount. This one call also checks allowance.
	let quote = sdk.fetch_pre_ocp_balance_and_est_fee_ncw(&user_id, &payment, false).await?;
	let coin = payment.coin;
	let total = quote.tot_amt_w_fee(&payment.amount, coin)?;
	let balance = parse_payment_amount(&quote.balance, coin)?;
	println!(
		"Pay {} {coin} to {} on {chain}. Estimated fee {} {coin}. Safe to spend {} {coin}.",
		payment.amount, payment.to_address, quote.est_fee, quote.balance
	);
	ensure!(total <= balance, "Insufficient {coin} balance for payment and fee");
	if permit2::needs_approval(chain, coin, &quote.allowance, total) {
		println!(
			"Confirm the token approval in the payer's wallet, then rerun to refresh preflight."
		);
		return Ok(());
	}
	println!("Existing Permit2 allowance covers this payment.");
	if !submit {
		return Ok(());
	}

	let signer = permit2::test_signer(payer)?;
	let amounts = [
		(coin, parse_payment_amount(&payment.amount, coin)?),
		(coin, parse_payment_amount(&quote.est_fee, coin)?),
	];
	let (spender, permissions) = permit2::expected_permissions(&amounts)?;
	let base = PayOnchainPayloadNcwBase {
		chain,
		coin,
		to_address: payment.to_address,
		amount: payment.amount,
		est_fee: quote.est_fee,
		deadline: now_timestamp_secs() + 5 * 60,
	};
	// Fetch a nonce only when ready to sign. Keep this base unchanged through submission.
	let permit = sdk.create_domain_permit_for_sig(&user_id, &base).await?;
	let (nonce, sig) =
		permit2::sign_permit(&signer, &permit, chain, spender, &permissions, base.deadline)?;
	let payload = PayOnchainPayloadNcw { base, nonce, sig, memo: payment.memo };
	let receipt_id = sdk
		.pay_onchain_ncw(&user_id, PayOnchainNcwRequest { payload, sched: None })
		.await
		.wrap_err(
			"Submission failed or status is unknown. Check payment history before trying again",
		)?;
	println!("Receipt ID {receipt_id}");
	Ok(())
}
