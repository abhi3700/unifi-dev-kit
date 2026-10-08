use crate::{Sdk, types::ApiHandler};
use unifi_sdk_primitives::{
	permit2::DomainPermitForSig,
	types::{
		BulkPayBase, BulkPayItem, BulkPayItemRequest, BulkPayResponse, ChainName, OcPayHistory,
		OcPayReceipt, PayHistoryFilterParams, PayOnchainNcwBulkRequest, PayOnchainNcwRequest,
		PayOnchainPayload, PayOnchainPayloadNcw, PayOnchainPayloadNcwBase, PayOnchainRequest,
		PreOcpPayload, PreOcpValues, PreOcpValuesNcw, PreOcpValuesNcwBulk, PreOcpValuesNcwParams,
		StableCoin,
	},
};

impl Sdk {
	/// Fetch user's pre-ocp net balance for a coin on a chain.
	pub async fn fetch_pre_ocp_net_balance(
		&self,
		user_id: &str,
		payload: PreOcpPayload,
	) -> eyre::Result<String> {
		let PreOcpPayload { coin, chain } = payload;
		let handler = ApiHandler::FetchPreOcpNetBalance;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			chain.to_string(),
			coin.to_string(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.get(url)).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Fetch the pre-ocp est. total fees for the given coin & chain in case of on-chain
	/// payment (OCP) by the user.
	pub async fn fetch_pre_ocp_total_est_fee(
		&self,
		user_id: &str,
		payload: PreOcpPayload,
	) -> eyre::Result<String> {
		let PreOcpPayload { coin, chain } = payload;
		let handler = ApiHandler::FetchPreOcpTotalEstFee;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			chain.to_string(),
			coin.to_string(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.get(url)).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Fetch the pre-ocp balance & est. total fees for the given coin & chain in case of on-chain
	/// payment (OCP) by the user.
	pub async fn fetch_pre_ocp_balance_and_est_fee(
		&self,
		user_id: &str,
		payload: PreOcpPayload,
	) -> eyre::Result<PreOcpValues> {
		let PreOcpPayload { coin, chain } = payload;
		let handler = ApiHandler::FetchPreOcpBalanceAndEstFee;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			chain.to_string(),
			coin.to_string(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.get(url)).send().await;

		Sdk::process_response::<PreOcpValues>(resp).await
	}

	/// Prepare one self-custodial payment by checking balance and allowance and estimating its fee.
	pub async fn fetch_pre_ocp_balance_and_est_fee_ncw(
		&self,
		user_id: &str,
		payload: &PayOnchainPayload,
		is_fee_incl: bool,
	) -> eyre::Result<PreOcpValuesNcw> {
		let handler = ApiHandler::FetchPreOcpBalanceAndEstFeeNcw;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), is_fee_incl.to_string()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).json(payload).send().await;

		Sdk::process_response::<PreOcpValuesNcw>(resp).await
	}

	/// Prepare a self-custodial bulk payment by checking balances and allowances and estimating
	/// its total fee.
	pub async fn fetch_pre_ocp_balance_and_est_fee_ncw_bulkpay(
		&self,
		user_id: &str,
		chain: ChainName,
		items: &[BulkPayItem],
		is_fee_incl: bool,
	) -> eyre::Result<PreOcpValuesNcwBulk> {
		let handler = ApiHandler::FetchPreOcpBalanceAndEstFeeNcwBulkPay;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), is_fee_incl.to_string()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let request = BulkPayItemRequest { chain, items: items.to_owned() };
		let resp = self.with_auth(handler, self.client.post(url)).json(&request).send().await;

		Sdk::process_response::<PreOcpValuesNcwBulk>(resp).await
	}

	/// Fetch the raw allowance, balance, gas, and price inputs used to calculate one
	/// self-custodial payment.
	pub async fn prefetch_ncw_balance_fee_params(
		&self,
		user_id: &str,
		payload: PreOcpPayload,
	) -> eyre::Result<PreOcpValuesNcwParams> {
		let PreOcpPayload { coin, chain } = payload;
		let handler = ApiHandler::PreFetchNcwBalanceFeeParams;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			chain.to_string(),
			coin.to_string(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.get(url)).send().await;

		Sdk::process_response::<PreOcpValuesNcwParams>(resp).await
	}

	/// Request Airdrop on testnet
	pub async fn request_faucet(
		&self,
		user_id: &str,
		coin: StableCoin,
		chain: ChainName,
		create_session: bool,
	) -> eyre::Result<()> {
		let handler = ApiHandler::RequestFaucet;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			coin.to_string(),
			chain.to_string(),
			create_session.to_string(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).send().await;

		Sdk::process_response::<()>(resp).await
	}

	/// Pay onchain from a UniFi-managed wallet.
	pub async fn pay_onchain(
		&self,
		user_id: &str,
		is_fee_incl: bool,
		request: PayOnchainRequest,
	) -> eyre::Result<String> {
		let handler = ApiHandler::PayOnchain;
		let path = handler.fill_path_ordered(&[user_id.to_string(), is_fee_incl.to_string()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).json(&request).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Pay a merchant from a UniFi-managed wallet.
	pub async fn pay_onchain_merchant(
		&self,
		user_id: &str,
		is_fee_incl: bool,
		session_id: &str,
		payload: PayOnchainPayload,
	) -> eyre::Result<String> {
		let handler = ApiHandler::PayOnchainMerchant;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			is_fee_incl.to_string(),
			session_id.to_owned(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).json(&payload).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Create the EIP-712 domain and Permit2 message for one self-custodial payment.
	pub async fn create_domain_permit_for_sig(
		&self,
		user_id: &str,
		base_payload: &PayOnchainPayloadNcwBase,
	) -> eyre::Result<DomainPermitForSig> {
		let handler = ApiHandler::CreateDomainPermitForSig;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).json(base_payload).send().await;

		Sdk::process_response::<DomainPermitForSig>(resp).await
	}

	/// Create the EIP-712 domain and Permit2 message for a self-custodial bulk payment.
	pub async fn create_domain_permit_for_sig_bulkpay(
		&self,
		user_id: &str,
		base_payload: &BulkPayBase,
	) -> eyre::Result<DomainPermitForSig> {
		let handler = ApiHandler::CreateDomainPermitForSigBulkPay;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).json(base_payload).send().await;

		Sdk::process_response::<DomainPermitForSig>(resp).await
	}

	/// Submit one signed self-custodial payment.
	pub async fn pay_onchain_ncw(
		&self,
		user_id: &str,
		request: PayOnchainNcwRequest,
	) -> eyre::Result<String> {
		let handler = ApiHandler::PayOnchainNcw;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).json(&request).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Submit a signed self-custodial merchant payment.
	pub async fn pay_onchain_merchant_ncw(
		&self,
		user_id: &str,
		session_id: &str,
		payload: PayOnchainPayloadNcw,
	) -> eyre::Result<String> {
		let handler = ApiHandler::PayOnchainMerchantNcw;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), session_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).json(&payload).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Submit a signed self-custodial bulk payment.
	pub async fn bulk_pay_ncw(
		&self,
		user_id: &str,
		request: PayOnchainNcwBulkRequest,
	) -> eyre::Result<BulkPayResponse> {
		let handler = ApiHandler::BulkPayNcw;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.post(url)).json(&request).send().await;

		Sdk::process_response::<BulkPayResponse>(resp).await
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
		let resp = self.with_auth(handler, self.client.get(url)).send().await;

		Sdk::process_response::<()>(resp).await
	}

	/// Return the receipt ID associated with a merchant payment session.
	///
	/// An empty string means that the session does not have a payment receipt yet.
	pub async fn get_payment_session_status(&self, session_id: &str) -> eyre::Result<String> {
		let handler = ApiHandler::GetPaymentSessionStatus;
		let path = handler.fill_path_ordered(&[session_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.get(url)).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// View onchain payment receipt
	pub async fn get_ocp_receipt(&self, receipt_id: &str) -> eyre::Result<OcPayReceipt> {
		let handler = ApiHandler::GetOcpReceipt;
		let path = handler.fill_path_ordered(&[receipt_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(handler, self.client.get(url)).send().await;

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
		self.get_ocp_receipts_with_cursor(user_id, sort_by_latest, from_start, filter, None)
			.await
	}

	/// View a page of onchain payment receipts using an explicit receipt cursor.
	///
	/// Use the last receipt ID from the current page when loading the next page and the first
	/// receipt ID when loading the previous page.
	pub async fn get_ocp_receipts_with_cursor(
		&self,
		user_id: &str,
		sort_by_latest: bool,
		from_start: bool,
		filter: Option<PayHistoryFilterParams>,
		cursor: Option<&str>,
	) -> eyre::Result<OcPayHistory> {
		let handler = ApiHandler::GetOcpReceipts;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			sort_by_latest.to_string(),
			from_start.to_string(),
		])?;
		let mut url = format!("{}{}", self.api_base_url, path);
		let mut query = Vec::new();

		if let Some(filter) = filter {
			let PayHistoryFilterParams { chain, status, limit, next_or_previous } = filter;

			if let Some(chain) = chain {
				query.push(format!("chain={chain}"));
			}

			if let Some(status) = status {
				query.push(format!("status={status}"));
			}

			if let Some(limit) = limit {
				query.push(format!("limit={limit}"));
			}

			if let Some(next_or_previous) = next_or_previous {
				query.push(format!("next_or_previous={next_or_previous}"));
			}
		}
		if let Some(cursor) = cursor {
			query.push(format!("cursor={cursor}"));
		}
		if !query.is_empty() {
			url.push('?');
			url.push_str(&query.join("&"));
		}

		let resp = self.with_auth(handler, self.client.get(url)).send().await;

		Sdk::process_response::<OcPayHistory>(resp).await
	}
}
