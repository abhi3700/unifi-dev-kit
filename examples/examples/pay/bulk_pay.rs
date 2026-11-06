//! Example: Submit a same-chain batch from a UniFi-managed wallet.
//!
//! The managed-wallet API handles approval and signing. The client sends one chain plus a payment
//! batch and receives one receipt ID for each recipient.

use colored::Colorize;
use unifi_examples::{init_sdk, take_input, with_spinner};
use unifi_sdk_primitives::types::{
	BulkPayItem, ChainName, Memo, PayOnchainCwBulkRequest, StableCoin,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let sdk = init_sdk();
	with_spinner(
		spinoff::spinners::Dots.into(),
		"🩺 Checking API health...".to_string(),
		sdk.health_check(),
		Some("✅ API is healthy!".to_string()),
		true,
	)
	.await
	.unwrap_or_else(|error| panic!("{}", error.to_string().red().bold()));

	let user_id = take_input("Enter a valid User ID: ")?;
	let chain = ChainName::Sepolia;
	let request = PayOnchainCwBulkRequest {
		chain,
		batch: vec![
			BulkPayItem {
				coin: StableCoin::USDT,
				to_address: "0x2222222222222222222222222222222222222222".to_string(),
				amount: "5".to_string(),
				memo: Memo::BulkPay,
			},
			BulkPayItem {
				coin: StableCoin::USDC,
				to_address: "0x3333333333333333333333333333333333333333".to_string(),
				amount: "12.5".to_string(),
				memo: Memo::BulkPay,
			},
		],
		scheds: None,
	};

	let coins = request.batch.iter().map(|item| item.coin).collect::<Vec<_>>();
	let preflight = with_spinner(
		spinoff::spinners::Dots.into(),
		"⏳ Fetching managed-wallet balances and fee inputs...".to_string(),
		sdk.fetch_pre_ocp_balance_and_est_fee_bulkpay(&user_id, chain, &coins, false),
		Some("✅ Batch preflight loaded!".to_string()),
		true,
	)
	.await
	.unwrap_or_else(|error| panic!("{}", error.to_string().red().bold()));

	for entry in preflight.coin_entries {
		println!(
			"{}",
			format!("👛 {} safe to spend: {}", entry.coin, entry.balance).green().bold()
		);
	}

	let response = with_spinner(
		spinoff::spinners::Dots.into(),
		"🚀 Submitting managed bulk payment...".to_string(),
		sdk.bulk_pay_cw(&user_id, false, &request),
		Some("✅ Batch submitted!".to_string()),
		true,
	)
	.await
	.unwrap_or_else(|error| panic!("{}", error.to_string().red().bold()));

	println!("{}", format!("📦 Bulk payment ID: {}", response.bulk_pay_id).green().bold());
	for (index, receipt_id) in response.receipt_ids.iter().enumerate() {
		println!("{}", format!("🧾 Payment {} receipt: {receipt_id}", index + 1).green().bold());
	}

	Ok(())
}
