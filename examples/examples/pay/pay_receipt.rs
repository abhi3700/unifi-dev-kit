//! Pay Receipt

use colored::Colorize;
use unifi_examples::{display_pay_receipt, init_sdk, take_input, with_spinner};

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

	let receipt_id = &take_input("Enter a valid Pay Receipt ID: ")?;

	println!("================================================");

	let pay_receipt = with_spinner(
		spinoff::spinners::Dots.into(),
		"⏳ Fetching receipt...".to_string(),
		sdk.get_ocp_receipt(receipt_id),
		Some("✅ Fetching receipt done!".to_string()),
		true,
	)
	.await?;

	println!("\n{}", "✅ Payment Receipt".green().bold());
	display_pay_receipt(pay_receipt);

	Ok(())
}
