use super::Sdk;
use crate::types::ApiHandler;
use std::collections::HashMap;
use unifi_sdk_primitives::types::{
	ChainName, ChainProtocol, StableCoin, WalletBalancesByChain, WalletBalancesByCoin,
};

impl Sdk {
	pub async fn get_user_wallet_address(
		&self,
		user_id: &str,
		chain: ChainName,
	) -> eyre::Result<String> {
		let handler = ApiHandler::GetUserWalletAddress;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), chain.to_string()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;
		Sdk::process_response::<String>(resp).await
	}

	pub async fn get_user_wallet_addresses(
		&self,
		user_id: &str,
	) -> eyre::Result<HashMap<ChainProtocol, String>> {
		let handler = ApiHandler::GetUserWalletAddresses;
		let path = handler.fill_path_ordered(&[user_id.to_owned()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;
		Sdk::process_response::<HashMap<ChainProtocol, String>>(resp).await
	}

	/// Get a chain's coin balance for a given user
	pub async fn get_oc_chain_coin_balance(
		&self,
		user_id: &str,
		chain: ChainName,
		coin: StableCoin,
	) -> eyre::Result<String> {
		let handler = ApiHandler::GetOcChainCoinBalance;
		let path = handler.fill_path_ordered(&[
			user_id.to_owned(),
			chain.to_string(),
			coin.to_string(),
		])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<String>(resp).await
	}

	/// Get a chain's all coins balances for a given user
	pub async fn get_oc_chain_all_coins_balances(
		&self,
		user_id: &str,
		chain: ChainName,
	) -> eyre::Result<HashMap<StableCoin, String>> {
		let handler = ApiHandler::GetOcChainAllCoinsBalances;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), chain.to_string()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<HashMap<StableCoin, String>>(resp).await
	}

	/// Get user's wallet balances on a chain
	///
	/// ## Usage
	/// - Show all the values in Web App's wallet page by chain.
	pub async fn get_wallet_balances_by_chain(
		&self,
		user_id: &str,
		chain: ChainName,
	) -> eyre::Result<WalletBalancesByChain> {
		let handler = ApiHandler::GetWalletBalancesByChain;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), chain.to_string()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<WalletBalancesByChain>(resp).await
	}

	/// Get user's wallet balances for a coin
	///
	/// ## Usage
	/// - Show all the values in Web App's wallet page by coin.
	pub async fn get_wallet_balances_by_coin(
		&self,
		user_id: &str,
		coin: StableCoin,
	) -> eyre::Result<WalletBalancesByCoin> {
		let handler = ApiHandler::GetWalletBalancesByCoin;
		let path = handler.fill_path_ordered(&[user_id.to_owned(), coin.to_string()])?;
		let url = format!("{}{}", self.api_base_url, path);
		let resp = self.with_auth(self.client.get(url)).send().await;

		Sdk::process_response::<WalletBalancesByCoin>(resp).await
	}
}
