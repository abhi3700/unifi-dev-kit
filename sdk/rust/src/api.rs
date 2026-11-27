use crate::{Sdk, types::ApiHandler};
use unifi_sdk_primitives::types::Api;

impl Sdk {
	/// Read the API plan, remaining credits, and key summaries for the API key owner's account.
	///
	/// Pass the key owner's UniFi `user_id`. Each call costs one API credit, and
	/// `metadata.credits` is the balance after that charge.
	///
	/// Check the balance periodically or before a batch of requests. If it falls below your
	/// threshold, alert your agents so they can pause or defer work until credits are available.
	/// The request fails if the account has already exhausted its API credits.
	pub async fn get_api(&self, user_id: &str) -> eyre::Result<Api> {
		let handler = ApiHandler::GetApi;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_endpoint_base_url, path);
		let resp = self.with_auth(handler, self.client.get(url)).send().await;

		Sdk::process_response::<Api>(resp).await
	}
}
