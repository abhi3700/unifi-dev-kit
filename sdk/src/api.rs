use crate::{Sdk, types::ApiHandler};
use unifi_sdk_primitives::types::Api;

impl Sdk {
	pub async fn generate_api_key(&self, user_id: &str, name: &str) -> eyre::Result<String> {
		let handler = ApiHandler::GenerateApiKey;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), name.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).send().await;

		Sdk::process_response::<String>(resp).await
	}

	pub async fn get_api(&self, user_id: &str) -> eyre::Result<Api> {
		let handler = ApiHandler::GetApi;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.get(url)).send().await;

		Sdk::process_response::<Api>(resp).await
	}

	pub async fn del_api_key(&self, user_id: &str, name: &str) -> eyre::Result<()> {
		let handler = ApiHandler::DelApiKey;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), name.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.delete(url)).send().await;

		Sdk::process_response::<()>(resp).await
	}
}
