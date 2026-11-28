//! Read one contacts page. Reuse the same filters with the returned cursor.

use eyre::OptionExt;
use unifi_examples::init_sdk;
use unifi_sdk_primitives::types::{ChainProtocol, ContactListParams};

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let mut args = std::env::args().skip(1);
	let user_id = args.next().ok_or_eyre("Usage: contacts USER_ID [CURSOR]")?;
	let params = ContactListParams {
		protocol: Some(ChainProtocol::Evm),
		limit: Some(10),
		cursor: args.next(),
		..Default::default()
	};
	let page = init_sdk().get_all_contacts(&user_id, &params).await?;
	for contact in page.contacts {
		println!("{}\t{}", contact.name, contact.address);
	}
	if let Some(cursor) = page.next_cursor {
		println!("Next cursor {cursor}");
	}
	Ok(())
}
