//! View wallet addresses across chain protocols.

use colored::*;
use unifi_examples::{init_sdk, take_input, with_spinner};

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let sdk = init_sdk();
	sdk.health_check().await?;

	println!("================================================");

	let user_id = &take_input("Enter a valid User ID: ")?;

	println!("================================================");

	let addresses = with_spinner(
		spinoff::spinners::Dots.into(),
		format!("Fetching wallet addresses ").yellow().to_string(),
		sdk.get_user_wallet_addresses(user_id),
		Some(format!("Wallet addresses: ").bold().to_string()),
		true,
	)
	.await?;
	println!("{}", format!("{:#?}", addresses).green().bold());

	Ok(())
}
