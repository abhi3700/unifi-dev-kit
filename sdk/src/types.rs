use crate::errors::OmniPayError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ApiHandler {
	/* Health */
	HealthCheck,
	Healthz,

	/* API Key for User */
	GenerateApiKey,
	GetApi,
	DelApiKey,

	/* Auth */
	CreateNcAuthChallenge,
	GenerateJwtForNc,

	/* Profile */
	SetUserProfile,
	GetUserProfile,

	/* Contacts */
	AddUserContacts,
	DelUserContact,
	UpdateUserContact,
	GetUserContacts,
	GetUserContactByUid,
	GetUserContactsByName,

	/* Payment Onchain */
	FetchPreOcpNetBalance,
	FetchPreOcpTotalEstFee,
	FetchPreOcpBalanceAndEstFee,
	FetchPreOcpBalanceAndEstFeeNcw,
	FetchPreOcpBalanceAndEstFeeNcwBulkPay,
	PreFetchNcwBalanceFeeParams,
	RequestFaucet,
	PayOnchain,
	PayOnchainMerchant,
	PayOnchainMerchantNcw,
	GetPaymentSessionStatus,
	CreateDomainPermitForSig,
	CreateDomainPermitForSigBulkPay,
	PayOnchainNcw,
	BulkPayNcw,
	FliqNotifyPayer,
	GetOcpReceipt,
	GetOcpReceipts,

	/* Wallet */
	GetUserWalletAddress,
	GetUserWalletAddresses,
	// Onchain
	GetOcChainCoinBalance,
	GetOcChainAllCoinsBalances,
	GetWalletBalances,
	GetWalletBalancesByChain,
	GetWalletBalancesByCoin,

	/* Session */
	CreateSession,
	GetSession,
}

impl ApiHandler {
	pub(crate) fn path(&self) -> &'static str {
		use ApiHandler as AH;
		match self {
			/* Health */
			AH::Healthz => "/healthz",
			AH::HealthCheck => "/health",

			/* API Key for User */
			AH::GenerateApiKey => "/api_key/generate/{user_id}/{name}",
			AH::GetApi => "/api/{user_id}",
			AH::DelApiKey => "/api_key/del/{user_id}/{name}",

			/* Auth */
			AH::CreateNcAuthChallenge => "/auth/challenge/{address}",
			AH::GenerateJwtForNc => "/auth/generate_jwt/{address}",

			/* Profile */
			AH::SetUserProfile | AH::GetUserProfile => "/profile/{user_id}",

			/* Contacts */
			AH::GetUserContacts | AH::AddUserContacts => "/contacts/{user_id}",
			AH::DelUserContact | AH::UpdateUserContact | AH::GetUserContactByUid =>
				"/contacts/{user_id}/{uid}",
			AH::GetUserContactsByName => "/contacts/by_name/{user_id}/{name}",

			/* Wallet */
			AH::GetUserWalletAddress => "/wallet/address/{user_id}/{chain}",
			AH::GetUserWalletAddresses => "/wallet/addresses/{user_id}",
			// Onchain
			AH::GetOcChainCoinBalance => "/wallet/onchain/balance/{user_id}/{chain}/{coin}",
			AH::GetOcChainAllCoinsBalances => "/wallet/onchain/balances/{user_id}/{chain}",
			AH::GetWalletBalances => "/wallet/balances/{user_id}",
			AH::GetWalletBalancesByChain => "/wallet/balances/by_chain/{user_id}/{chain}",
			AH::GetWalletBalancesByCoin => "/wallet/balances/by_coin/{user_id}/{coin}",

			/* Payment Onchain */
			AH::FetchPreOcpNetBalance => "/payment/onchain/net_balance/{user_id}/{chain}/{coin}",
			AH::FetchPreOcpTotalEstFee => "/payment/onchain/est_fee/{user_id}/{chain}/{coin}",
			AH::FetchPreOcpBalanceAndEstFee =>
				"/payment/onchain/balance_est_fee/{user_id}/{chain}/{coin}",
			AH::FetchPreOcpBalanceAndEstFeeNcw =>
				"/payment/onchain/balance_est_fee_ncw/{user_id}/{is_fee_incl}",
			AH::FetchPreOcpBalanceAndEstFeeNcwBulkPay =>
				"/payment/onchain/balance_est_fee_ncw_bulkpay/{user_id}/{is_fee_incl}",
			AH::PreFetchNcwBalanceFeeParams =>
				"/payment/onchain/ncw/pre_fetch_balance_est_fee_params/{user_id}/{chain}/{coin}",
			AH::RequestFaucet => "/faucet/{user_id}/{coin}/{chain}/{create_session}",
			AH::PayOnchain => "/payment/onchain/{user_id}/{is_fee_incl}",
			AH::PayOnchainMerchant =>
				"/payment/onchain/merchant/{user_id}/{is_fee_incl}/{session_id}",
			AH::PayOnchainMerchantNcw => "/payment/onchain/ncw/merchant/{user_id}/{session_id}",
			AH::GetPaymentSessionStatus => "/payment/merchant/session/{session_id}",
			AH::CreateDomainPermitForSig => "/payment/onchain/ncw/create_domain_permit/{user_id}",
			AH::CreateDomainPermitForSigBulkPay =>
				"/payment/onchain/ncw/create_domain_permit_bulkpay/{user_id}",
			AH::PayOnchainNcw => "/payment/onchain/ncw/{user_id}",
			AH::BulkPayNcw => "/payment/onchain/ncw_bulkpay/{user_id}",
			AH::FliqNotifyPayer =>
				"/payment/onchain/fliq/notify/payer/{pid}/{chain}/{coin}/{to_address}/{amount}",
			AH::GetOcpReceipt => "/payment/onchain/receipt/{receipt_id}",
			AH::GetOcpReceipts =>
				"/payment/onchain/receipts/{user_id}/{sort_by_latest}/{from_start}",

			AH::CreateSession => "/session/create/{user_id}",
			AH::GetSession => "/session/{session_id}",
		}
	}

	/// Replaces `{}`-wrapped placeholders in a given path using the provided parameters,
	/// assuming the order of the parameters corresponds exactly to the order of placeholders.
	///
	/// # Arguments
	/// - `template`: a path with placeholders (e.g. "/user/{coin}/{chain}")
	/// - `params`: a slice of values to substitute in order
	///
	/// # Returns
	/// A new path string with placeholders replaced
	pub(crate) fn fill_path_ordered(&self, params: &[String]) -> eyre::Result<String> {
		let template = self.path();
		let mut filled_path = String::new();
		let mut i = 0;
		let mut param_index = 0;

		while let Some(start) = template[i..].find('{') {
			let abs_start = i + start;
			if let Some(end) = template[abs_start..].find('}') {
				let abs_end = abs_start + end;
				// Push text before placeholder
				filled_path.push_str(&template[i..abs_start]);
				// Push replacement value
				if let Some(val) = params.get(param_index) {
					filled_path.push_str(val);
					param_index += 1;
				} else {
					return Err(OmniPayError::LessParamsForApiPath.into());
				}
				i = abs_end + 1;
			} else {
				return Err(OmniPayError::UnclosedPlaceholderInApiPathTemplate.into());
			}
		}

		// Ensure there are no extra params
		if param_index != params.len() {
			return Err(OmniPayError::MoreParamsForApiPath.into());
		}

		// Push remaining part
		filled_path.push_str(&template[i..]);
		Ok(filled_path)
	}

	pub(crate) fn is_apikey_required(&self) -> bool {
		use ApiHandler as AH;
		match self {
			// Signature verification before this api call.
			AH::CreateNcAuthChallenge | AH::GenerateJwtForNc => false,
			_ => true,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::ApiHandler as AH;

	#[test]
	fn public_api_paths_match_the_openapi_surface() {
		let expected = [
			(AH::Healthz, "/healthz"),
			(AH::HealthCheck, "/health"),
			(AH::GetUserWalletAddress, "/wallet/address/{user_id}/{chain}"),
			(AH::GetUserWalletAddresses, "/wallet/addresses/{user_id}"),
			(AH::GetOcChainCoinBalance, "/wallet/onchain/balance/{user_id}/{chain}/{coin}"),
			(AH::GetOcChainAllCoinsBalances, "/wallet/onchain/balances/{user_id}/{chain}"),
			(AH::GetWalletBalances, "/wallet/balances/{user_id}"),
			(AH::GetWalletBalancesByChain, "/wallet/balances/by_chain/{user_id}/{chain}"),
			(AH::GetWalletBalancesByCoin, "/wallet/balances/by_coin/{user_id}/{coin}"),
			(AH::FetchPreOcpNetBalance, "/payment/onchain/net_balance/{user_id}/{chain}/{coin}"),
			(AH::FetchPreOcpTotalEstFee, "/payment/onchain/est_fee/{user_id}/{chain}/{coin}"),
			(
				AH::FetchPreOcpBalanceAndEstFee,
				"/payment/onchain/balance_est_fee/{user_id}/{chain}/{coin}",
			),
			(
				AH::FetchPreOcpBalanceAndEstFeeNcw,
				"/payment/onchain/balance_est_fee_ncw/{user_id}/{is_fee_incl}",
			),
			(
				AH::FetchPreOcpBalanceAndEstFeeNcwBulkPay,
				"/payment/onchain/balance_est_fee_ncw_bulkpay/{user_id}/{is_fee_incl}",
			),
			(
				AH::PreFetchNcwBalanceFeeParams,
				"/payment/onchain/ncw/pre_fetch_balance_est_fee_params/{user_id}/{chain}/{coin}",
			),
			(AH::PayOnchain, "/payment/onchain/{user_id}/{is_fee_incl}"),
			(
				AH::PayOnchainMerchant,
				"/payment/onchain/merchant/{user_id}/{is_fee_incl}/{session_id}",
			),
			(AH::CreateDomainPermitForSig, "/payment/onchain/ncw/create_domain_permit/{user_id}"),
			(
				AH::CreateDomainPermitForSigBulkPay,
				"/payment/onchain/ncw/create_domain_permit_bulkpay/{user_id}",
			),
			(AH::PayOnchainNcw, "/payment/onchain/ncw/{user_id}"),
			(AH::BulkPayNcw, "/payment/onchain/ncw_bulkpay/{user_id}"),
			(AH::PayOnchainMerchantNcw, "/payment/onchain/ncw/merchant/{user_id}/{session_id}"),
			(AH::GetPaymentSessionStatus, "/payment/merchant/session/{session_id}"),
			(AH::GetOcpReceipt, "/payment/onchain/receipt/{receipt_id}"),
			(
				AH::GetOcpReceipts,
				"/payment/onchain/receipts/{user_id}/{sort_by_latest}/{from_start}",
			),
		];

		assert_eq!(expected.len(), 25);
		for (handler, path) in expected {
			assert_eq!(handler.path(), path);
		}
	}
}
