//! Example: Request testnet funds on Sepolia.
//!
//! Need test tokens?
//! Get some 𝓯𝓻𝓮𝓮 tokens 🪙 to pay 💸 someone 👤 on the Sepolia (Testnet) chain! 🎉
//!
//! Just pick a stablecoin (USDT or USDC or DAI) and run. 🚀

use colored::Colorize;
use unifi_examples::{init_sdk, take_input, with_spinner};
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
	.await?;

	println!("================================================");

	let user_id = &take_input("Enter a valid User ID: ")?;
	let selected_coin = StableCoin::USDC;
	println!("Selected coin: {selected_coin}");

	println!("================================================");

	if let Err(e) = with_spinner(
		spinoff::spinners::Dots.into(),
		"⏳ Requesting faucet 💧...".to_string(),
		sdk.request_faucet(user_id, selected_coin, ChainName::Sepolia),
		Some("✅ Faucet requested!".to_string()),
		true,
	)
	.await
	{
		println!("{}", e.to_string().red().bold());
	} else {
		println!(
			"{}",
			format!("🚀 Faucet request sent! 100 {} will be deposited shortly...", selected_coin)
				.green()
				.bold()
		);
	}

	Ok(())
}
