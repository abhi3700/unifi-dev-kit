//! Example: Fetch net_balance & est. fee
//!
//! NOTE: The est. fee may include approval cost, if not approved to BundlePay contract.

use colored::Colorize;
use unifi_examples::{init_sdk, take_input, with_spinner};
use unifi_sdk_primitives::types::{ChainName, PreOcpPayload, PreOcpValues, StableCoin};

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
	.unwrap_or_else(|e| panic!("{}", e.to_string().red().bold()));

	println!("================================================");

	let user_id = &take_input("Enter a valid User ID: ")?;
	let selected_chain = ChainName::Sepolia;
	println!("Selected chain: {selected_chain}");
	let selected_coin = StableCoin::USDC;
	println!("Selected coin: {selected_coin}");

	println!("================================================");

	let PreOcpValues { is_coin_allowance_zero, balance: net_balance, est_fee } = with_spinner(
		spinoff::spinners::Dots.into(),
		"⏳ Fetching balance & fees...".to_string(),
		sdk.fetch_pre_ocp_balance_and_est_fee(
			user_id,
			PreOcpPayload { coin: selected_coin, chain: selected_chain },
		),
		Some("✅ Balance & fee check done!".to_string()),
		true,
	)
	.await
	.unwrap_or_else(|e| panic!("{}", e.to_string().red().bold()));

	println!("{}", format!("👛 Net balance: {}", net_balance).green().bold());
	println!("{}", format!("💸 Estimated fee: {}", est_fee).green().bold());

	if is_coin_allowance_zero {
		println!(
			"{}",
			format!("⚠️ Est. fee include {selected_coin} approval cost.")
				.bright_yellow()
				.bold()
		);
	}

	Ok(())
}
