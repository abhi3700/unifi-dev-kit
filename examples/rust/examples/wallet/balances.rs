//! Example: View wallet balances on chain.

use colored::*;
use unifi_examples::{
	init_sdk, print_balances_by_chain, print_balances_by_coin, take_input, with_spinner,
};
use unifi_sdk_primitives::types::{ChainName, StableCoin};

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

	let selected_chain = ChainName::Sepolia;
	let selected_coin = StableCoin::USDT;

	println!("================================================");

	let user_id = &take_input("Enter a valid User ID: ")?;

	println!("================================================");

	// TODO: will uncomment this once auth module integrated into SDK.
	// verify_totp(&sdk, user_id).await?;

	let balances = with_spinner(
		spinoff::spinners::Dots.into(),
		format!("Get balances on {selected_chain}:").yellow().to_string(),
		sdk.get_wallet_balances_by_chain(user_id, selected_chain),
		Some(format!("Get balances on {selected_chain}:").bold().to_string()),
		true,
	)
	.await
	.unwrap_or_else(|e| panic!("{}", e.to_string().red().bold()));

	print_balances_by_chain(selected_chain, &balances);
	println!("------------------------------------------------");

	let balances = with_spinner(
		spinoff::spinners::Dots.into(),
		format!("Get balances of {selected_coin}:").yellow().to_string(),
		sdk.get_wallet_balances_by_coin(user_id, selected_coin),
		Some(format!("Get balances of {selected_coin}:").bold().to_string()),
		true,
	)
	.await
	.unwrap_or_else(|e| panic!("{}", e.to_string().red().bold()));
	print_balances_by_coin(selected_coin, &balances);

	Ok(())
}
