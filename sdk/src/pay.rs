use crate::{Sdk, types::ApiHandler};
use unifi_sdk_primitives::types::{
	ChainName, OcPayHistory, OcPayReceipt, PayHistoryFilterParams, PayOnchainPayload,
	PreOcpPayload, PreOcpValues, StableCoin,
};

impl Sdk {
	/// Fetch user's pre-transaction net onchain balance for a coin on a chain.
	pub async fn fetch_pre_ocp_net_onchain_balance(
		&self,
		user_id: &str,
		payload: PreOcpPayload,
	) -> eyre::Result<String> {
		let PreOcpPayload { coin, chain } = payload;
		let handler = ApiHandler::FetchPreOcpNetOnchainBalance;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			chain.to_string(),
			coin.to_string(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Fetch the pre-ocp est. total fees for the given coin & chain in case of on-chain
	/// payment (OCP) by the user.
	pub async fn fetch_pre_ocp_total_est_fees(
		&self,
		user_id: &str,
		payload: PreOcpPayload,
	) -> eyre::Result<String> {
		let PreOcpPayload { coin, chain } = payload;
		let handler = ApiHandler::FetchPreOcpTotalEstFees;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			chain.to_string(),
			coin.to_string(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Fetch the pre-ocp balance & est. total fees for the given coin & chain in case of on-chain
	/// payment (OCP) by the user.
	pub async fn fetch_pre_ocp_balance_and_est_fees(
		&self,
		user_id: &str,
		payload: PreOcpPayload,
	) -> eyre::Result<PreOcpValues> {
		let PreOcpPayload { coin, chain } = payload;
		let handler = ApiHandler::FetchPreOcpBalanceAndEstFees;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			chain.to_string(),
			coin.to_string(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<PreOcpValues>(resp).await
	}

	/// Request Airdrop on testnet
	pub async fn request_faucet(
		&self,
		user_id: &str,
		coin: StableCoin,
		chain: ChainName,
	) -> eyre::Result<()> {
		let handler = ApiHandler::RequestFaucet;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			coin.to_string(),
			chain.to_string(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.post(url)).send().await;

		Sdk::process_response::<()>(resp).await
	}

	/// Pay onchain
	pub async fn pay_onchain(
		&self,
		user_id: &str,
		is_fee_incl: bool,
		payload: PayOnchainPayload,
	) -> eyre::Result<String> {
		let handler = ApiHandler::PayOnchain;
		let path = handler.fill_path_ordered(&[user_id.to_string(), is_fee_incl.to_string()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.post(url)).json(&payload).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Notify FliQ Payer.
	pub async fn fliq_notify_payer(
		&self,
		pid: &str,
		payload: PayOnchainPayload,
	) -> eyre::Result<()> {
		let PayOnchainPayload { chain, coin, to_address, amount, .. } = payload;
		let handler = ApiHandler::FliqNotifyPayer;
		let path = handler.fill_path_ordered(&[
			pid.to_owned(),
			chain.to_string(),
			coin.to_string(),
			to_address,
			amount,
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<()>(resp).await
	}

	/// View onchain payment receipt
	pub async fn get_ocp_receipt(&self, receipt_id: &str) -> eyre::Result<OcPayReceipt> {
		let handler = ApiHandler::GetOcpReceipt;
		let path = handler.fill_path_ordered(&[receipt_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<OcPayReceipt>(resp).await
	}

	/// View onchain payment receipts for a user_id
	pub async fn get_ocp_receipts(
		&self,
		user_id: &str,
		sort_by_latest: bool,
		from_start: bool,
		filter: Option<PayHistoryFilterParams>,
	) -> eyre::Result<OcPayHistory> {
		let handler = ApiHandler::GetOcpReceipts;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			sort_by_latest.to_string(),
			from_start.to_string(),
		])?;
		let mut url = format!("{}{}", self.api_base_url, path);

		// Add filters
		if let Some(filter) = filter {
			let PayHistoryFilterParams { chain, status, limit, next_or_previous } = filter;

			url.push('?');

			if let Some(chain) = chain {
				url.push_str(&format!("chain={}&", chain));
			}

			if let Some(status) = status {
				url.push_str(&format!("status={}&", status));
			}

			if let Some(limit) = limit {
				url.push_str(&format!("limit={}&", limit));
			}

			if let Some(next_or_previous) = next_or_previous {
				url.push_str(&format!("next_or_previous={}&", next_or_previous));
			}
		}

		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<OcPayHistory>(resp).await
	}
}
