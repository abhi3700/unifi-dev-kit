//! Pay History
//!
//! Here, during each run, it loads from page-1 & then it goes to next page & so on so forth.
//!
//! NOTE: During each full example run, the pointer is brought back from the last page to first
//! page.
//!
//! In terms of UI:
//! 1. load the page-1
//! 2. User if scrolls-up more even after reaching end, then loads the next page & then if further
//!    scrolls-up, then loads next page & likewise it keeps going.

use colored::Colorize;
use unifi_examples::{init_sdk, take_input};
use unifi_sdk_primitives::types::{OcPayHistory, PayHistoryFilterParams};

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let sdk = init_sdk();
	sdk.health_check().await?;

	println!("================================================");

	let user_id = &take_input("Enter a valid User ID: ")?;

	println!("================================================");

	// load payment history (by latest) i.e. page-1 & then automatically Next next... unless there
	// is no next receipts.
	println!("========================= Page-1 =========================");
	let OcPayHistory { receipts, .. } = sdk.get_ocp_receipts(user_id, true, true, None).await?;
	println!("{}", format!("{:#?}", receipts).green().bold());

	let mut count = receipts.len();
	while let Ok(next_page) = sdk
		.get_ocp_receipts(
			user_id,
			true,
			false,
			Some(PayHistoryFilterParams { next_or_previous: Some(true), ..Default::default() }),
		)
		.await
	{
		println!("========================= NEXT ... =========================");
		// NOTE: on purpose, didn't use receipts field as in the display I want to see each page's
		// Prev & Next values.
		println!("{}", format!("{:#?}", next_page).green().bold());
		count += next_page.receipts.len();

		if !next_page.has_next {
			break;
		}
	}

	println!("{}", format!("Total receipts: {}", count).bold().blue());

	Ok(())
}
