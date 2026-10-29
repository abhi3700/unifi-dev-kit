use crate::{Sdk, types::ApiHandler};
use unifi_sdk_primitives::types::{
	AddContactMainPayload, Contact, ContactListParams, ContactPage, UpdateContactPayload,
};

impl Sdk {
	pub async fn get_user_contacts(&self, user_id: &str) -> eyre::Result<ContactPage> {
		self.get_user_contacts_page(user_id, &ContactListParams::default()).await
	}

	pub async fn get_user_contacts_page(
		&self,
		user_id: &str,
		params: &ContactListParams,
	) -> eyre::Result<ContactPage> {
		let handler = ApiHandler::GetUserContacts;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let response = self
			.with_auth(
				handler,
				self.client.get(format!("{}{}", self.api_base_url, path)).query(params),
			)
			.send()
			.await;
		Sdk::process_response::<ContactPage>(response).await
	}

	pub async fn get_user_contact(&self, user_id: &str, contact_id: &str) -> eyre::Result<Contact> {
		let handler = ApiHandler::GetUserContactById;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), contact_id.to_owned()])?;
		let response = self
			.with_auth(handler, self.client.get(format!("{}{}", self.api_base_url, path)))
			.send()
			.await;
		Sdk::process_response::<Contact>(response).await
	}

	pub async fn add_user_contacts(
		&self,
		user_id: &str,
		payload: AddContactMainPayload,
	) -> eyre::Result<()> {
		let handler = ApiHandler::AddUserContacts;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let response = self
			.with_auth(handler, self.client.post(format!("{}{}", self.api_base_url, path)))
			.json(&payload)
			.send()
			.await;
		Sdk::process_response::<()>(response).await
	}

	pub async fn update_user_contact(
		&self,
		user_id: &str,
		contact_id: &str,
		payload: &UpdateContactPayload,
	) -> eyre::Result<()> {
		let handler = ApiHandler::UpdateUserContact;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), contact_id.to_owned()])?;
		let response = self
			.with_auth(handler, self.client.put(format!("{}{}", self.api_base_url, path)))
			.json(payload)
			.send()
			.await;
		Sdk::process_response::<()>(response).await
	}

	pub async fn delete_user_contact(&self, user_id: &str, contact_id: &str) -> eyre::Result<()> {
		let handler = ApiHandler::DelUserContact;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), contact_id.to_owned()])?;
		let response = self
			.with_auth(handler, self.client.delete(format!("{}{}", self.api_base_url, path)))
			.send()
			.await;
		Sdk::process_response::<()>(response).await
	}
}
