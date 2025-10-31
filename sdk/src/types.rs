use crate::errors::OmniPayError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ApiHandler {
	/* Health */
	HealthCheck,
	Healthz,

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
	FetchPreOcpNetOnchainBalance,
	FetchPreOcpTotalEstFees,
	FetchPreOcpBalanceAndEstFees,
	RequestFaucet,
	PayOnchain,
	FliqNotifyPayer,
	GetOcpReceipt,
	GetOcpReceipts,

	/* Wallet */
	GetUserWalletAddress,
	GetUserWalletAddresses,
	// Onchain
	GetOcChainCoinBalance,
	GetOcChainAllCoinsBalances,
	GetWalletBalancesByChain,
	GetWalletBalancesByCoin,
}

// #[cfg(any(feature = "types"))]
impl ApiHandler {
	pub fn path(&self) -> &'static str {
		use ApiHandler as AH;
		match self {
			/* Health */
			AH::Healthz => "/healthz",
			AH::HealthCheck => "/health",

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
			AH::GetWalletBalancesByChain => "/wallet/balances/by_chain/{user_id}/{chain}",
			AH::GetWalletBalancesByCoin => "/wallet/balances/by_coin/{user_id}/{coin}",

			/* Payment Onchain */
			AH::FetchPreOcpNetOnchainBalance =>
				"/payment/onchain/net_balance/{user_id}/{chain}/{coin}",
			AH::FetchPreOcpTotalEstFees => "/payment/onchain/est_fee/{user_id}/{chain}/{coin}",
			AH::FetchPreOcpBalanceAndEstFees =>
				"/payment/onchain/balance_est_fees/{user_id}/{chain}/{coin}",
			AH::RequestFaucet => "/faucet/{user_id}/{coin}/{chain}",
			AH::PayOnchain => "/payment/onchain/{user_id}/{is_fee_incl}",
			AH::FliqNotifyPayer =>
				"/payment/onchain/fliq/notify/payer/{pid}/{chain}/{coin}/{to_address}/{amount}",
			AH::GetOcpReceipt => "/payment/onchain/receipt/{receipt_id}",
			AH::GetOcpReceipts =>
				"/payment/onchain/receipts/{user_id}/{sort_by_latest}/{from_start}",
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
	pub fn fill_path_ordered(&self, params: &[String]) -> eyre::Result<String> {
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
}
