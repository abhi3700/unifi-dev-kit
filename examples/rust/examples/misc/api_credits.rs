//! Check the API key owner's plan and the credits left after this read.

use eyre::OptionExt;
use unifi_examples::init_sdk;

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let owner = std::env::args().nth(1).ok_or_eyre("Usage: api_credits KEY_OWNER_USER_ID")?;
	let account = init_sdk().get_api(&owner).await?;
	println!(
		"Plan {:?}. Remaining API credits {}.",
		account.metadata.plan, account.metadata.credits
	);
	Ok(())
}
