//! View wallet balances on chain.

use colored::*;
use unifi_examples::{init_sdk, take_input, with_spinner};
use unifi_sdk_primitives::types::{ChainName, StableCoin};

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let sdk = init_sdk();
	sdk.health_check().await?;

	let selected_chain = ChainName::Sepolia;
	let selected_coin = StableCoin::USDT;

	println!("================================================");

	let user_id = &take_input("Enter a valid User ID: ")?;

	println!("================================================");

	// TODO: will uncomment this once auth module integrated into SDK.
	// verify_totp(&sdk, user_id).await?;

	let balances = with_spinner(
		spinoff::spinners::Dots.into(),
		format!("Get balances on-chain").yellow().to_string(),
		sdk.get_wallet_balances_by_chain(user_id, selected_chain),
		Some(format!("Get balances on-chain").bold().to_string()),
		true,
	)
	.await?;
	println!("{}", format!("{:#?}", balances).green().bold());
	println!("------------------------------------------------");

	let balances = with_spinner(
		spinoff::spinners::Dots.into(),
		format!("Get balances of coin").yellow().to_string(),
		sdk.get_wallet_balances_by_coin(user_id, selected_coin),
		Some(format!("Get balances of coin").bold().to_string()),
		true,
	)
	.await?;
	println!("{}", format!("{:#?}", balances).green().bold());

	Ok(())
}
