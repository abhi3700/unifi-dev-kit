use bson::{
	Bson::{self, Document as BsonDocument},
	doc,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

/// Modes for running in diff. cases
///
/// Details in README -- "Running Modes" section.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
	Test,
	Dev,
	Prod,
}

impl FromStr for Mode {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"test" => Ok(Mode::Test),
			"dev" => Ok(Mode::Dev),
			"prod" => Ok(Mode::Prod),
			_ => Err(format!("Invalid mode: {}. Allowed values: \"test\", \"dev\", \"prod\".", s)),
		}
	}
}

impl Mode {
	pub fn is_test(&self) -> bool {
		self.eq(&Mode::Test)
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ChainProtocol {
	Evm,
	// Tron,
	// Solana,
	// Near,
	// Eos,
	// Substrate,
}

impl From<ChainProtocol> for String {
	fn from(val: ChainProtocol) -> Self {
		val.to_string()
	}
}

impl Display for ChainProtocol {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl AsRef<str> for ChainProtocol {
	fn as_ref(&self) -> &str {
		match self {
			ChainProtocol::Evm => "Evm",
		}
	}
}

impl ChainProtocol {
	pub const fn all() -> &'static [ChainProtocol] {
		use ChainProtocol as C;
		&[C::Evm /* , Tron, Solana, Near, Eos, Substrate */]
	}
}

// More would be added later.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum ChainName {
	#[default]
	Ethereum,
	Polygon,
	// BnbChain,
	// Base,
	// Arbitrum,
	// Optimism,
	// PolygonZkEvm,
	// OpBNB,
	Sepolia,
	/// For local testing
	Anvil,
}

impl From<ChainName> for String {
	fn from(val: ChainName) -> Self {
		val.to_string()
	}
}

impl Display for ChainName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

/// Used because of use in dioxus route params.
impl FromStr for ChainName {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use ChainName::*;

		match s.to_lowercase().as_str() {
			"ethereum" => Ok(Ethereum),
			"polygon" => Ok(Polygon),
			"sepolia" => Ok(Sepolia),
			"anvil" => Ok(Anvil),
			_ => Err(format!("Invalid chain name: {}", s)),
		}
	}
}

impl AsRef<str> for ChainName {
	fn as_ref(&self) -> &str {
		use ChainName::*;
		match self {
			Ethereum => "Ethereum",
			Polygon => "Polygon",
			Sepolia => "Sepolia",
			Anvil => "Anvil",
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Copy, Default)]
pub enum StableCoin {
	#[default]
	USDT,
	USDC,
	DAI,
}

impl From<StableCoin> for Bson {
	fn from(val: StableCoin) -> Self {
		bson::Bson::String(val.to_string())
	}
}

impl<'de> Deserialize<'de> for StableCoin {
	fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		use StableCoin::*;
		let s = String::deserialize(deserializer)?;
		match s.as_str() {
			"USDT" => Ok(USDT),
			"USDC" => Ok(USDC),
			"DAI" => Ok(DAI),
			_ => Err(serde::de::Error::unknown_variant(&s, &["USDT", "USDC", "DAI"])),
		}
	}
}

impl From<StableCoin> for String {
	fn from(val: StableCoin) -> Self {
		val.to_string()
	}
}

impl From<Coin> for StableCoin {
	fn from(val: Coin) -> Self {
		use Coin as C;
		match val {
			C::USDT => StableCoin::USDT,
			C::USDC => StableCoin::USDC,
			C::DAI => StableCoin::DAI,
			_ => panic!("Unsupported coin: {:?} as StableCoin", val),
		}
	}
}

impl Display for StableCoin {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl AsRef<str> for StableCoin {
	fn as_ref(&self) -> &str {
		use StableCoin as S;
		match self {
			S::USDT => "USDT",
			S::USDC => "USDC",
			S::DAI => "DAI",
		}
	}
}

/// Used because of use in dioxus route params.
impl FromStr for StableCoin {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use StableCoin as S;
		match s.to_uppercase().as_str() {
			"USDT" => Ok(S::USDT),
			"USDC" => Ok(S::USDC),
			"DAI" => Ok(S::DAI),
			_ => Err(format!("Invalid stablecoin: {}", s)),
		}
	}
}

impl StableCoin {
	pub fn all() -> &'static [StableCoin] {
		use StableCoin as S;
		&[S::USDT, S::USDC, S::DAI]
	}

	pub fn decimals(&self) -> u8 {
		use StableCoin as S;
		match self {
			S::USDT => 6,
			S::USDC => 6,
			S::DAI => 18,
		}
	}

	pub fn from_strr(stablecoin: &str) -> eyre::Result<Self> {
		use StableCoin as S;
		match stablecoin.to_uppercase().as_str() {
			"USDT" => Ok(S::USDT),
			"USDC" => Ok(S::USDC),
			"DAI" => Ok(S::DAI),
			_ => Err(eyre::eyre!(format!("Invalid stablecoin: {}", stablecoin))),
		}
	}
}

/// All coins (network/gas + stablecoins) supported by OmniPay
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Coin {
	ETH,
	MATIC,
	// NOTE: For Sepolia testnet, use `Eth'
	USDT,
	USDC,
	DAI,
}

impl From<StableCoin> for Coin {
	fn from(val: StableCoin) -> Self {
		use StableCoin as S;
		match val {
			S::USDT => Coin::USDT,
			S::USDC => Coin::USDC,
			S::DAI => Coin::DAI,
		}
	}
}

impl Display for Coin {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

/// ## Usage
/// - In Web App, wallet page (by chain)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct WalletBalancesByChain {
	/// total value in USD
	pub total_usd: String,
	/// Coin details:
	/// - price
	/// - balance
	/// - value_usd
	pub coin_details: Vec<(StableCoin, WalletBalancesByChainCoinDetails)>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct WalletBalancesByChainCoinDetails {
	/// price in USD
	pub price_usd: String,
	/// formatted balance (in 2 decimals)
	pub balance: String,
	/// value in USD = price_usd * balance.
	pub value_usd: String,
}

/// ## Usage
/// - In Web App, wallet page (by coin)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct WalletBalancesByCoin {
	/// total value in USD
	pub total_usd: String,
	/// price in USD
	pub price_usd: String,
	/// Chain details:
	/// - balance
	/// - value_usd
	pub chain_details: Vec<(ChainName, WalletBalancesByCoinChainDetails)>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct WalletBalancesByCoinChainDetails {
	/// formatted balance (in 2 decimals)
	pub balance: String,
	/// value in USD = price_usd * balance.
	pub value_usd: String,
}

/// Shows only user profile
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct UserProfile {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub nickname: Option<String>,
	/// if email set means email is verified as it might have been verified from the client side.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub email: Option<String>,
	// pub is_email_verified: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub preferred_fiat_currency: Option<Currency>,
}

impl From<UserProfile> for Bson {
	fn from(val: UserProfile) -> Self {
		BsonDocument(doc! {
			"name": val.name,
			"nickname": val.nickname,
			"email": val.email,
			"preferred_fiat_currency": val.preferred_fiat_currency,
		})
	}
}

impl Default for UserProfile {
	fn default() -> Self {
		UserProfile {
			name: None,
			nickname: None,
			email: None,
			preferred_fiat_currency: Some(Currency::USD),
		}
	}
}

/* Currency */

#[allow(non_camel_case_types)]
#[derive(Default, Serialize, Debug, Clone, PartialEq, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
	#[default]
	USD,
	INR,
	EUR,
	// Add other currencies as needed
}

impl From<Currency> for Bson {
	fn from(val: Currency) -> Self {
		Bson::String(val.to_string())
	}
}

impl Display for Currency {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl<'de> Deserialize<'de> for Currency {
	fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		use Currency::*;

		let s = String::deserialize(deserializer)?.to_uppercase();
		match s.as_str() {
			"USD" => Ok(USD),
			"INR" => Ok(INR),
			"EUR" => Ok(EUR),
			_ => Err(serde::de::Error::unknown_variant(&s, &["USD", "INR", "EUR"])),
		}
	}
}

impl Currency {
	pub const fn all() -> &'static [Currency] {
		use Currency as C;
		&[C::USD, C::INR, C::EUR]
	}
}

/* OC Pay history */

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PayHistoryFilterParams {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub chain: Option<ChainName>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status: Option<OcPayReceiptStatus>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub limit: Option<i64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub next_or_previous: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OcPayHistory {
	pub has_prev: bool,
	pub receipts: Vec<OcPayReceipt>,
	pub has_next: bool,
}

/* OCP Receipt */

/// OCP Receipt
///
/// ## Usage
/// For SDK to retrieve the receipt from API response.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct OcPayReceipt {
	pub id: String,
	pub user_id: String,
	pub chain: ChainName,
	pub coins: Vec<StableCoin>,
	pub to_addresses: Vec<String>,
	pub amounts: Vec<String>,
	pub memo: Memo,
	// NOTE: not needed for now. In future, might put it back to see a bundle status or something.
	// Although we store this field in DB as present in `models::OcPayReceiptDoc`.
	// pub bundle_ids: Vec<i64>,
	pub tx_hash: String,
	pub tx_hashes: Vec<String>,
	pub status: OcPayReceiptStatus,
	pub start_ts_us: i64,
	pub end_ts_us: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum OcPayReceiptStatus {
	Failed,
	#[default]
	Processing,
	Completed,
}

impl Display for OcPayReceiptStatus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl From<OcPayReceiptStatus> for String {
	fn from(val: OcPayReceiptStatus) -> Self {
		val.to_string()
	}
}

impl From<OcPayReceiptStatus> for Bson {
	fn from(status: OcPayReceiptStatus) -> Self {
		Bson::String(status.to_string())
	}
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum Memo {
	#[default]
	General,
	SubscribeApi(ApiPlan, PaidPlanDuration),
	FliqPay,
}

impl From<Memo> for String {
	fn from(memo: Memo) -> Self {
		use Memo::*;
		match memo {
			General => "General".to_string(),
			SubscribeApi(plan, duration) =>
				format!("SubscribeApi:{}:{}", plan.as_ref(), duration.as_ref()),
			FliqPay => "FliqPay".to_string(),
		}
	}
}

impl FromStr for Memo {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use Memo::*;

		let parts: Vec<&str> = s.split(':').collect();
		match parts.as_slice() {
			["General"] => Ok(General),
			["FliqPay"] => Ok(FliqPay),
			["SubscribeApi", plan_str, duration_str] => {
				let plan =
					ApiPlan::from_str(plan_str).map_err(|e| format!("Invalid ApiPlan: {}", e))?;
				let duration = PaidPlanDuration::from_str(duration_str)
					.map_err(|e| format!("Invalid PaidPlanDuration: {}", e))?;
				Ok(SubscribeApi(plan, duration))
			},
			_ => Err(format!("Invalid Memo string: {}", s)),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
pub enum ApiPlan {
	#[default]
	Free,
	Starter,
	Growth,
	Scale,
	/// Custom
	Enterprise,
}

impl FromStr for ApiPlan {
	type Err = eyre::ErrReport;

	fn from_str(s: &str) -> eyre::Result<Self, Self::Err> {
		use ApiPlan as A;
		match s.to_lowercase().as_str() {
			"free" => Ok(A::Free),
			"starter" => Ok(A::Starter),
			"growth" => Ok(A::Growth),
			"scale" => Ok(A::Scale),
			"enterprise" => Ok(A::Enterprise),
			_ => Err(eyre::eyre!("Invalid API subscription plan: {}.", s)),
		}
	}
}

impl Display for ApiPlan {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl AsRef<str> for ApiPlan {
	fn as_ref(&self) -> &str {
		use ApiPlan as A;
		match self {
			A::Free => "Free",
			A::Starter => "Starter",
			A::Growth => "Growth",
			A::Scale => "Scale",
			A::Enterprise => "Enterprise",
		}
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash, Copy, Default)]
pub enum PaidPlanDuration {
	#[default]
	Month,
	Quarter,
	HalfYear,
	Year,
}

impl Display for PaidPlanDuration {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl AsRef<str> for PaidPlanDuration {
	fn as_ref(&self) -> &str {
		use PaidPlanDuration as P;
		match self {
			P::Month => "Month",
			P::Quarter => "Quarter",
			P::HalfYear => "HalfYear",
			P::Year => "Year",
		}
	}
}

impl FromStr for PaidPlanDuration {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use PaidPlanDuration as P;
		match s.to_lowercase().as_str() {
			"month" => Ok(P::Month),
			"quarter" => Ok(P::Quarter),
			"halfyear" => Ok(P::HalfYear),
			"year" => Ok(P::Year),
			_ => Err(format!("Invalid PaidPlanDuration: {}", s)),
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PreOcpPayload {
	pub coin: StableCoin,
	pub chain: ChainName,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PreOcpValues {
	pub is_coin_allowance_zero: bool,
	pub balance: String,
	pub est_fees: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, /* Props, */ PartialEq, Default)]
pub struct PayOnchainPayload {
	pub chain: ChainName,
	pub coin: StableCoin,
	pub to_address: String,
	pub amount: String,
	pub memo: Memo,
}

impl Display for PayOnchainPayload {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}
