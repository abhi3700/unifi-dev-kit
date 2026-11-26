use crate::{Sdk, types::ApiHandler};
use unifi_sdk_primitives::types::{
	Contact, ContactBatch, ContactBatchListParams, ContactBatchViewPage, ContactListParams,
	ContactPage, DeleteContactBatchPayload, DeleteContactsImpact, DeleteContactsPayload,
	UpsertContactsPayload, UpsertContactsResult,
};

impl Sdk {
	pub async fn get_all_contacts(
		&self,
		user_id: &str,
		params: &ContactListParams,
	) -> eyre::Result<ContactPage> {
		let handler = ApiHandler::GetUserContacts;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let response = self
			.with_auth(
				handler,
				self.client.get(format!("{}{}", self.api_endpoint_base_url, path)).query(params),
			)
			.send()
			.await;
		Sdk::process_response::<ContactPage>(response).await
	}

	pub async fn get_user_contact(&self, user_id: &str, contact_id: &str) -> eyre::Result<Contact> {
		let handler = ApiHandler::GetUserContactById;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), contact_id.to_owned()])?;
		let response = self
			.with_auth(handler, self.client.get(format!("{}{}", self.api_endpoint_base_url, path)))
			.send()
			.await;
		Sdk::process_response::<Contact>(response).await
	}

	pub async fn upsert_contacts(
		&self,
		user_id: &str,
		payload: UpsertContactsPayload,
	) -> eyre::Result<UpsertContactsResult> {
		let handler = ApiHandler::UpsertContacts;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let response = self
			.with_auth(handler, self.client.post(format!("{}{}", self.api_endpoint_base_url, path)))
			.json(&payload)
			.send()
			.await;
		Sdk::process_response::<UpsertContactsResult>(response).await
	}

	pub async fn delete_contacts(
		&self,
		user_id: &str,
		contact_ids: Vec<String>,
	) -> eyre::Result<DeleteContactsImpact> {
		let handler = ApiHandler::DeleteContacts;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let response = self
			.with_auth(
				handler,
				self.client.delete(format!("{}{}", self.api_endpoint_base_url, path)),
			)
			.json(&DeleteContactsPayload { contact_ids })
			.send()
			.await;
		Sdk::process_response::<DeleteContactsImpact>(response).await
	}

	pub async fn preview_delete_contacts(
		&self,
		user_id: &str,
		contact_ids: Vec<String>,
	) -> eyre::Result<DeleteContactsImpact> {
		let handler = ApiHandler::GetUserContacts;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_endpoint_base_url, path);
		let response = self
			.with_auth(handler, self.client.post(format!("{url}/delete_impact")))
			.json(&DeleteContactsPayload { contact_ids })
			.send()
			.await;
		Sdk::process_response::<DeleteContactsImpact>(response).await
	}

	pub async fn get_contact_batches(
		&self,
		user_id: &str,
		params: &ContactBatchListParams,
	) -> eyre::Result<ContactBatchViewPage> {
		let handler = ApiHandler::GetContactBatches;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let response = self
			.with_auth(
				handler,
				self.client.get(format!("{}{}", self.api_endpoint_base_url, path)).query(params),
			)
			.send()
			.await;
		Sdk::process_response::<ContactBatchViewPage>(response).await
	}

	pub async fn get_contact_batch(
		&self,
		user_id: &str,
		batch_id: &str,
	) -> eyre::Result<ContactBatch> {
		let handler = ApiHandler::GetContactBatch;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), batch_id.to_owned()])?;
		let response = self
			.with_auth(handler, self.client.get(format!("{}{}", self.api_endpoint_base_url, path)))
			.send()
			.await;
		Sdk::process_response::<ContactBatch>(response).await
	}

	pub async fn delete_contacts_batch(
		&self,
		user_id: &str,
		batch_id: &str,
		del_its_contacts: bool,
	) -> eyre::Result<()> {
		let handler = ApiHandler::DelContactBatch;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), batch_id.to_owned()])?;
		let response = self
			.with_auth(
				handler,
				self.client.delete(format!("{}{}", self.api_endpoint_base_url, path)),
			)
			.json(&DeleteContactBatchPayload { del_its_contacts })
			.send()
			.await;
		Sdk::process_response::<()>(response).await
	}
}
