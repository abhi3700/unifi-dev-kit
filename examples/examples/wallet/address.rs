//! View wallet address on a given chain.

use colored::*;
use unifi_examples::{init_sdk, take_input, with_spinner};
use unifi_sdk_primitives::types::ChainName;

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

	let selected_chain = ChainName::Sepolia;

	println!("================================================");

	let user_id = &take_input("Enter a valid User ID: ")?;

	println!("================================================");

	let address = with_spinner(
		spinoff::spinners::Dots.into(),
		format!("Fetching wallet address for {selected_chain}").yellow().to_string(),
		sdk.get_user_wallet_address(user_id, selected_chain),
		Some(format!("{selected_chain}'s Wallet address: ").bold().to_string()),
		true,
	)
	.await?;
	println!("{}", format!("{:#?}", address).green().bold());

	Ok(())
}
