//! Pay two recipients on one chain with one fee quote, one permit, and one submission.

mod permit2;

use eyre::{Context, OptionExt, bail, ensure};
use unifi_examples::init_sdk;
use unifi_sdk_primitives::{
	types::{BulkPayBase, Memo, PayOnchainNcwBulkRequest, PayOnchainPayloadNcwBulk, StableCoin},
	utils::{now_timestamp_secs, parse_payment_amount},
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let (args, submit) = permit2::args();
	let [payer, chain, to1, amount1, coin1, to2, amount2, coin2] = args.as_slice() else {
		bail!(
			"Usage: self_custodial_bulk_pay PAYER CHAIN RECIPIENT_1 AMOUNT_1 COIN_1 RECIPIENT_2 AMOUNT_2 COIN_2 [--submit]"
		);
	};
	let payer = permit2::payer_address(payer)?;
	let user_id = payer.to_string();
	let chain = permit2::test_chain(chain)?;
	let batch = vec![
		permit2::payment_item(payer, to1, amount1, coin1, Memo::BulkPay)?,
		permit2::payment_item(payer, to2, amount2, coin2, Memo::BulkPay)?,
	];
	for item in &batch {
		println!("Pay {} {} to {} on {chain}.", item.amount, item.coin, item.to_address);
	}

	let sdk = init_sdk();
	let quote = sdk
		.fetch_pre_ocp_balance_and_est_fee_ncw_bulkpay(&user_id, chain, &batch, false)
		.await?;
	let fee_coin = batch[1].coin;
	println!("Estimated batch fee {} {fee_coin}.", quote.tot_est_fee);
	// The last item's coin pays the fee. Repeated coins share one total and allowance.
	let totals = quote.tot_amt_w_fee2(&batch)?;
	let coins = StableCoin::dedup(batch.iter().map(|item| item.coin));
	let mut needs_approval = false;
	for coin in coins {
		let entry = quote.coin_entries.get(&coin).ok_or_eyre("Missing coin in batch quote")?;
		let total = totals[&coin];
		let balance = parse_payment_amount(&entry.balance, coin)?;
		println!("Safe to spend {} {coin}.", entry.balance);
		ensure!(total <= balance, "Insufficient {coin} balance for batch");
		needs_approval |= permit2::needs_approval(chain, coin, &entry.allowance, total);
	}
	if needs_approval {
		println!(
			"Confirm the token approvals in the payer's wallet, then rerun to refresh preflight."
		);
		return Ok(());
	}
	println!("Existing Permit2 allowances cover this batch.");
	if !submit {
		return Ok(());
	}

	let signer = permit2::test_signer(payer)?;
	let mut amounts = Vec::with_capacity(batch.len() + 1);
	for item in &batch {
		amounts.push((item.coin, parse_payment_amount(&item.amount, item.coin)?));
	}
	amounts.push((fee_coin, parse_payment_amount(&quote.tot_est_fee, fee_coin)?));
	let (spender, permissions) = permit2::expected_permissions(&amounts)?;
	let base = BulkPayBase {
		chain,
		batch,
		tot_est_fee: quote.tot_est_fee,
		deadline: now_timestamp_secs() + 5 * 60,
	};
	// Keep payment order, fee, and deadline unchanged after requesting the permit.
	let permit = sdk.create_domain_permit_for_sig_bulkpay(&user_id, &base).await?;
	let (nonce, sig) =
		permit2::sign_permit(&signer, &permit, chain, spender, &permissions, base.deadline)?;
	let request = PayOnchainNcwBulkRequest {
		payload: PayOnchainPayloadNcwBulk { base, nonce, sig },
		save_batch: false,
		..Default::default()
	};
	let result = sdk.bulk_pay_ncw(&user_id, request).await.wrap_err(
		"Submission failed or status is unknown. Check payment history before trying again",
	)?;
	println!("Bulk payment ID {}", result.bulk_pay_id);
	for (index, receipt_id) in result.receipt_ids.iter().enumerate() {
		println!("Payment {} receipt ID {receipt_id}", index + 1);
	}
	Ok(())
}
