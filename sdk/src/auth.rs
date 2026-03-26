use crate::{Sdk, types::ApiHandler};
use unifi_sdk_primitives::types::NcJwtAuthPayload;

impl Sdk {
	/// Generate JWT for NC.
	pub async fn generate_jwt_for_nc(
		&self,
		address: &str,
		payload: NcJwtAuthPayload,
	) -> eyre::Result<String> {
		let handler = ApiHandler::GenerateJwtForNc;
		let path = handler.fill_path_ordered(&[address.to_string()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).json(&payload).send().await;

		Sdk::process_response::<String>(resp).await
	}
}
