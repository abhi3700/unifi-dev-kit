//! Example: Send a gasless stablecoin payment seamlessly using the UniFi SDK.
//!
//! Prerequisites:
//! - You (the payer) must be registered on UniFi
//! - Sufficient balance available on the selected chain
//!
//! Inputs used in this example:
//! - user_id
//! - payload:
//!   - chain        (network to send on)
//!   - coin         (stablecoin type)
//!   - to_address   (recipient address)
//!   - amount       (token amount to send)

use colored::Colorize;
use unifi_examples::{init_sdk, take_input, with_spinner};
use unifi_sdk_primitives::{
	types::{ChainName, PayOnchainPayload, PreOcpPayload, PreOcpValues, StableCoin},
	utils::validate_and_parse_amount,
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
	.await?;

	println!("================================================");

	let user_id = &take_input("Enter a valid User ID: ")?;
	let selected_chain = ChainName::Sepolia;
	println!("Selected chain: {selected_chain}");
	let selected_coin = StableCoin::USDC;
	println!("Selected coin: {selected_coin}");
	let to_address = "0xDA741C58b3e299A8c51Aa80DF70AB2881d17499c";
	println!("Payee address: {to_address}");
	let amount = "10.124";
	println!("amount: {amount}");

	println!("================================================");

	let PreOcpValues { is_coin_allowance_zero, balance: net_balance, est_fees } = with_spinner(
		spinoff::spinners::Dots.into(),
		"⏳ Fetching balance & fees...".to_string(),
		sdk.fetch_pre_ocp_balance_and_est_fees(
			user_id,
			PreOcpPayload { coin: selected_coin, chain: selected_chain },
		),
		Some("✅ Balance & fee check done!".to_string()),
		true,
	)
	.await?;

	println!("{}", format!("💸 Estimated fees: {}", est_fees).green().bold());
	println!("{}", format!("👛 Net balance: {}", net_balance).green().bold());
	if is_coin_allowance_zero {
		println!(
			"{}",
			format!("⚠️ Est. fees include {selected_coin} approval cost.")
				.bright_yellow()
				.bold()
		);
	}
	if validate_and_parse_amount(amount, selected_coin, &net_balance, &est_fees).is_err() {
		println!(
			"{}",
			"Insufficient balance 💰.\nPlease 📩 deposit or request faucet (on Sepolia testnet)"
				.red()
				.bold()
		);
	}

	// ==================== Submit payment ==================================
	// For exclusive payment
	let is_fee_incl = false;
	let receipt_id = with_spinner(
		spinoff::spinners::Dots.into(),
		"🚀 Processing payment ⏳...".to_string(),
		sdk.pay_onchain(
			user_id,
			is_fee_incl,
			PayOnchainPayload {
				chain: selected_chain,
				coin: selected_coin,
				to_address: to_address.to_owned(),
				amount: amount.to_owned(),
				memo: unifi_sdk_primitives::types::Memo::General,
			},
		),
		Some("✅ Payment submitted!".to_string()),
		true,
	)
	.await?;
	println!("{}", format!("🧾 Receipt ID: {}", receipt_id).green().bold());
	println!("{}", "🚚 Track this payment using the receipt ID above 👆.".cyan());

	Ok(())
}
