//! Example: View wallet addresses across chain protocols.

use colored::*;
use unifi_examples::{init_sdk, take_input, with_spinner};

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

	println!("================================================");

	let addresses = with_spinner(
		spinoff::spinners::Dots.into(),
		"Fetching wallet addresses ".to_string().yellow().to_string(),
		sdk.get_user_wallet_addresses(user_id),
		Some("Wallet addresses: ".to_string().bold().to_string()),
		true,
	)
	.await
	.unwrap_or_else(|e| panic!("{}", e.to_string().red().bold()));
	println!("{}", format!("{:#?}", addresses).green().bold());

	Ok(())
}
