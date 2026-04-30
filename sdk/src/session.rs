use crate::{Sdk, types::ApiHandler};
use unifi_sdk_primitives::types::{FalconSessionParams, UserSession};

impl Sdk {
	/// Create session id for web app from `user_id`.
	pub async fn create_session(
		&self,
		user_id: &str,
		request: FalconSessionParams,
	) -> eyre::Result<String> {
		let handler = ApiHandler::CreateSession;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).json(&request).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Get session data by session_id.
	pub async fn get_session(&self, session_id: &str) -> eyre::Result<UserSession> {
		let handler = ApiHandler::GetSession;
		let path = handler.fill_path_ordered(&[session_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.get(url)).send().await;

		Sdk::process_response::<UserSession>(resp).await
	}
}
