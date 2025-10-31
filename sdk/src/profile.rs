use super::Sdk;
use crate::types::ApiHandler;
use unifi_sdk_primitives::types::UserProfile;

impl Sdk {
	pub async fn set_user_profile(
		&self,
		user_id: &str,
		user_profile: &UserProfile,
	) -> eyre::Result<()> {
		let handler = ApiHandler::SetUserProfile;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.post(url)).json(&user_profile).send().await;

		Sdk::process_response::<()>(resp).await
	}

	pub async fn get_user_profile(&self, user_id: &str) -> eyre::Result<UserProfile> {
		let handler = ApiHandler::GetUserProfile;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<UserProfile>(resp).await
	}
}
