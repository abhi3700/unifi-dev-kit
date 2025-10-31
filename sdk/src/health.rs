use crate::{Sdk, types::ApiHandler};

impl Sdk {
	pub async fn healthz(&self) -> eyre::Result<String> {
		let handler = ApiHandler::Healthz;
		let url = format!("{}{}", self.api_base_url, handler.path());
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<String>(resp).await
	}

	pub async fn health_check(&self) -> eyre::Result<String> {
		let handler = ApiHandler::HealthCheck;
		let url = format!("{}{}", self.api_base_url, handler.path());
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<String>(resp).await
	}
}
