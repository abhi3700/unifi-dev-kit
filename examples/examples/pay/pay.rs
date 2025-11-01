//! Pay History

use colored::Colorize;
use unifi_examples::{init_sdk, take_input};
use unifi_sdk_primitives::{
	types::{ChainName, StableCoin},
	utils::is_value_gte,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let sdk = init_sdk();
	sdk.health_check().await?;

	println!("================================================");

	let user_id = &take_input("Enter a valid User ID: ")?;
	let selected_chain = ChainName::Sepolia;
	let selected_coin = StableCoin::USDC;

	println!("================================================");

	let available_balance =
		sdk.get_oc_chain_coin_balance(user_id, selected_chain, selected_coin).await?;
	if !is_value_gte(&available_balance, "50", selected_coin.decimals())? {
		sdk.request_faucet(user_id, selected_coin, selected_chain).await?;

		// Wait for balance update after faucet request
		while let Ok(oc_balance) =
			sdk.get_oc_chain_coin_balance(user_id, selected_chain, selected_coin).await
		{
			/* Proceed further for OCP (i.e. exit from loop), if you have some balance */
			if is_value_gte(&oc_balance, "50", selected_coin.decimals())? {
				break;
			}

			println!("=== Faucet is still processing... Retrying in 5 seconds");
			tokio::time::sleep(std::time::Duration::from_secs(5)).await;
		}
	}

	Ok(())
}
