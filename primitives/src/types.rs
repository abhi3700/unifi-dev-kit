use crate::errors::UfiError;
#[cfg(feature = "utils")]
use crate::utils::{now_timestamp_secs, sanitize_and_parse_amount};
#[cfg(feature = "utils")]
use alloy_primitives::U256;
use alloy_primitives::{Address, address};
use bson::{
	Bson::{self, Document as BsonDocument},
	doc,
};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, str::FromStr};

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
#[derive(
	Archive,
	RkyvSerialize,
	RkyvDeserialize,
	Serialize,
	Deserialize,
	Debug,
	PartialEq,
	Eq,
	Hash,
	Clone,
	Copy,
	PartialOrd,
	Ord,
	Default,
)]
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

impl FromStr for ChainName {
	type Err = String;

	fn from_str(chain: &str) -> Result<Self, Self::Err> {
		use ChainName as C;

		match chain.to_lowercase().as_str() {
			"ethereum" => Ok(C::Ethereum),
			"polygon" => Ok(C::Polygon),
			"sepolia" => Ok(C::Sepolia),
			"anvil" => Ok(C::Anvil),
			_ => Err(format!("Invalid chain name: {}", chain)),
		}
	}
}

impl AsRef<str> for ChainName {
	fn as_ref(&self) -> &str {
		use ChainName as C;
		match self {
			C::Ethereum => "Ethereum",
			C::Polygon => "Polygon",
			C::Sepolia => "Sepolia",
			C::Anvil => "Anvil",
		}
	}
}

pub struct GasEstimate {
	pub eth_transfer: u128,
	pub approve: u128,
	pub transfer_from: u128,
	pub permit_transfer_from: u128,
}

impl ChainName {
	pub fn all(mode: Mode) -> &'static [ChainName] {
		use ChainName as C;

		if matches!(mode, Mode::Test) {
			&[C::Ethereum, C::Polygon, C::Sepolia, C::Anvil]
		} else {
			&[C::Ethereum, C::Polygon, C::Sepolia]
		}
	}

	pub fn to_gas_coin(&self) -> Coin {
		use ChainName as C;
		match self {
			C::Ethereum | C::Sepolia | C::Anvil => Coin::ETH,
			C::Polygon => Coin::POL,
		}
	}

	/// Get the gas limit (max. feasible for prediction so that the actual gas consumed is lower
	/// than the predicted) of all the used functions (in onchain payment) for a given
	/// coin. Although all the ERC20 tokens have same gas usage irrespective of chains. But, just
	/// in case. That's why used [`Option`] type for coin. In future, based on stats we might have
	/// diff. gas values for (chain, coin) combo.
	pub fn get_gas_usage_limit(&self, coin: Option<StableCoin>) -> GasEstimate {
		// NOTE: currently, every token is ERC20 with same code. So, the gas estimate kept same.
		// TODO: Reduce the gas limit for `approve` and `transfer_from` later on depending on
		// the bulk users' data.

		let est_gas = GasEstimate {
			eth_transfer: 21_000,
			approve: 100_000,
			transfer_from: 80_000,
			permit_transfer_from: 120_000,
		};

		if let Some(coin) = coin {
			use StableCoin as S;
			match (coin, self) {
				(S::USDT, Self::Ethereum) => est_gas,
				(S::USDC, Self::Ethereum) => est_gas,
				(S::DAI, Self::Ethereum) => est_gas,
				(S::USDT, Self::Polygon) => est_gas,
				(S::USDC, Self::Polygon) => est_gas,
				(S::DAI, Self::Polygon) => est_gas,
				(S::USDT, Self::Sepolia) => est_gas,
				(S::USDC, Self::Sepolia) => est_gas,
				(S::DAI, Self::Sepolia) => est_gas,
				(S::USDT, Self::Anvil) => est_gas,
				(S::USDC, Self::Anvil) => est_gas,
				(S::DAI, Self::Anvil) => est_gas,
			}
		} else {
			est_gas
		}
	}

	/// Get Permit2 Contract address for supported chain
	pub fn get_permit2_sc_addr(&self) -> Address {
		use ChainName as C;
		let addr = address!("000000000022D473030F116dDEE9F6B43aC78BA3");
		match self {
			C::Ethereum => addr,
			C::Polygon => addr,
			C::Sepolia | C::Anvil => addr,
		}
	}
}

#[derive(
	Archive,
	RkyvSerialize,
	RkyvDeserialize,
	Debug,
	Clone,
	PartialEq,
	Eq,
	Hash,
	Serialize,
	Copy,
	PartialOrd,
	Ord,
	Default,
)]
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
		use StableCoin as S;
		let s = String::deserialize(deserializer)?;
		match s.as_str() {
			"USDT" => Ok(S::USDT),
			"USDC" => Ok(S::USDC),
			"DAI" => Ok(S::DAI),
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
}

/// All coins (network/gas + stablecoins) supported by OmniPay
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum Coin {
	ETH,
	POL,
	// NOTE: For Sepolia testnet, use `Eth'
	USDT,
	USDC,
	DAI,
}

impl Coin {
	pub fn decimals(&self) -> u8 {
		use Coin as C;

		match self {
			C::ETH => 18,
			C::POL => 18,
			C::USDT => StableCoin::USDT.decimals(),
			C::USDC => StableCoin::USDC.decimals(),
			C::DAI => StableCoin::DAI.decimals(),
		}
	}
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OcPayReceipt {
	pub id: String,
	pub entity: String,
	/// Can be used to detect if payer was custodial or non-custodial.
	pub user_id: String,
	pub is_fee_incl: bool,
	pub chain: ChainName,
	pub coin: StableCoin,
	pub to_address: String,
	pub amount: String,
	pub memo: Memo,
	pub est_fee: String,
	pub act_fee: String,
	pub tx_hash: String,
	/// Block number
	pub block_num: i64,
	pub status: OcPayReceiptStatus,
	pub start_ts_us: i64,
	/// timestamp @ `Confirmed` block status.
	pub end_ts_us: i64,
}

impl Default for OcPayReceipt {
	fn default() -> Self {
		Self {
			id: Default::default(),
			entity: Default::default(),
			user_id: Default::default(),
			is_fee_incl: Default::default(),
			chain: Default::default(),
			coin: Default::default(),
			to_address: Default::default(),
			amount: Default::default(),
			memo: Default::default(),
			est_fee: Self::default_est_fee(),
			act_fee: Self::default_act_fee(),
			tx_hash: Default::default(),
			block_num: Default::default(),
			status: Default::default(),
			start_ts_us: Default::default(),
			end_ts_us: Default::default(),
		}
	}
}

impl OcPayReceipt {
	pub fn default_est_fee() -> String {
		"0".to_owned()
	}
	pub fn default_act_fee() -> String {
		"0".to_owned()
	}

	/// This fn is used to hide the savings field in the Receipt UI.
	///
	/// `true` => legacy receipt
	pub fn is_legacy(&self) -> bool {
		self.est_fee.eq(&Self::default_est_fee()) && self.act_fee.eq(&Self::default_act_fee())
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum OcPayReceiptStatus {
	/// This is marked when Payment submitted to UniFi's sequencer.
	#[default]
	Processing,
	/// Payment submitted to UniFi's sequencer is found as invalid (insufficient balance, etc..). \
	/// NOTE: In case of NC op, this might happen. Checked via `scan_nc_op`.
	Failed,
	/// Payment submitted was sent to onchain i.e. added to a block & tx_hash generated.
	Confirmed,
	/// Completed payment is now finalized i.e. added to a block (with finalized tag now).
	Finalized,
	/// Completed payment is not added to canonical chain. \
	/// Hence, payer needs to retry the payment. \
	/// NOTE: Although chances of happening this is very low, but still considered here.
	Reorged,
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

impl OcPayReceiptStatus {
	pub fn is_processing(&self) -> bool {
		self.eq(&Self::Processing)
	}

	pub fn is_confirmed(&self) -> bool {
		self.eq(&Self::Confirmed)
	}

	pub fn is_finalized(&self) -> bool {
		self.eq(&Self::Finalized)
	}
}

#[derive(
	Archive,
	RkyvSerialize,
	RkyvDeserialize,
	Debug,
	Serialize,
	Deserialize,
	Default,
	Clone,
	PartialEq,
)]
pub enum Memo {
	#[default]
	General,
	SubscribeApi(ApiPlan, PaidPlanDuration),
	FliqPay,
	FliqPayMerchant,
	/// For Salary/Payroll, Vendor payments, ..
	BulkPay,
	Custom(String),
}

impl Memo {
	/// Max characters stored for a custom memo purpose to avoid receipt bloating.
	pub const CUSTOM_PURPOSE_MAX_LEN: usize = 48;

	fn normalize_custom_purpose(purpose: &str) -> String {
		purpose.trim().chars().take(Self::CUSTOM_PURPOSE_MAX_LEN).collect()
	}
}

impl Display for Memo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// Reuse canonical serialization logic from `From<Memo> for String` to ensure consistent
		// formatting (e.g., SubscribeApi:Starter:Month)
		let value: String = self.clone().into();
		write!(f, "{}", value)
	}
}

impl From<Memo> for String {
	fn from(memo: Memo) -> Self {
		use Memo::*;
		match memo {
			General => "General".to_string(),
			SubscribeApi(plan, duration) =>
				format!("SubscribeApi:{}:{}", plan.as_ref(), duration.as_ref()),
			FliqPay => "FliqPay".to_string(),
			FliqPayMerchant => "FliqPayMerchant".to_string(),
			BulkPay => "BulkPay".to_string(),
			Custom(purpose) => Memo::normalize_custom_purpose(&purpose),
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
			["FliqPayMerchant"] => Ok(FliqPayMerchant),
			["BulkPay"] => Ok(BulkPay),
			["SubscribeApi", plan_str, duration_str] => {
				let plan =
					ApiPlan::from_str(plan_str).map_err(|e| format!("Invalid ApiPlan: {}", e))?;
				let duration = PaidPlanDuration::from_str(duration_str)
					.map_err(|e| format!("Invalid PaidPlanDuration: {}", e))?;
				Ok(SubscribeApi(plan, duration))
			},
			[custom_purpose] if !custom_purpose.trim().is_empty() =>
				Ok(Custom(Memo::normalize_custom_purpose(custom_purpose))),
			_ => Err(format!("Invalid Memo string: {}", s)),
		}
	}
}

#[derive(
	Archive,
	RkyvSerialize,
	RkyvDeserialize,
	Debug,
	Serialize,
	Deserialize,
	Default,
	Clone,
	Copy,
	PartialEq,
)]
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

impl ApiPlan {
	pub fn is_free(&self) -> bool {
		self.eq(&ApiPlan::Free)
	}

	/// Returns the total credits quota/limit for the selected API plan for duration.
	///
	/// This value represents the maximum credits allocated for a user
	/// based on their plan. The `credits_left` field in `UserDocument`
	/// will be decremented from this total as API usage occurs.
	pub fn credit_quota(&self, duration: PaidPlanDuration) -> i64 {
		use ApiPlan as A;
		use PaidPlanDuration as D;

		// TODO: decide these no.s
		let monthly_quota = match self {
			A::Free => 100_000,
			A::Starter => 1_000_000,
			A::Growth => 20_000_000,
			A::Scale => 50_000_000,
			A::Enterprise => 700_000_000,
		};

		match duration {
			D::Month => monthly_quota,
			D::Quarter => monthly_quota * 4,
			D::HalfYear => monthly_quota * 6,
			D::Year => monthly_quota * 12,
		}
	}

	/// Returns the total image credits quota for the selected API plan for duration.
	///
	/// This value represents the maximum credits allocated for a user
	/// based on their plan. The `credits_left` field in `UserDocument`
	/// will be decremented from this total as API usage occurs.
	pub fn img_credit_quota(&self, duration: PaidPlanDuration) -> i64 {
		use ApiPlan as A;
		use PaidPlanDuration as D;

		let monthly_quota = match self {
			A::Free => 3,
			A::Starter => 500,
			A::Growth => 3_000,
			A::Scale => 15_000,
			A::Enterprise => 100_000,
		};

		match duration {
			D::Month => monthly_quota,
			D::Quarter => monthly_quota * 4,
			D::HalfYear => monthly_quota * 6,
			D::Year => monthly_quota * 12,
		}
	}
}

#[derive(
	Archive,
	RkyvSerialize,
	RkyvDeserialize,
	Serialize,
	Deserialize,
	Debug,
	Clone,
	Eq,
	PartialEq,
	Hash,
	Copy,
	Default,
)]
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

impl PaidPlanDuration {
	pub fn to_seconds(&self) -> i64 {
		let month_in_secs = 30 * 86_400;

		use PaidPlanDuration as P;
		match self {
			P::Month => month_in_secs,
			P::Quarter => 3 * month_in_secs,
			P::HalfYear => 6 * month_in_secs,
			P::Year => 12 * month_in_secs,
		}
	}
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct PreOcpPayload {
	pub coin: StableCoin,
	pub chain: ChainName,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PreOcpValues {
	pub is_coin_allowance_zero: bool,
	pub balance: String,
	pub est_fee: String,
}

/// Pre-OCP Values using NCW Params (for calc) for single pay.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PreOcpValuesNcw {
	/// Is coin allowance sufficient?
	/// NOTE: This field is redundant. But for direct use, added this field. Else, we have to
	/// convert required_allowance to U256 to check if it's zero.
	pub is_suff: bool,
	/// User need to approve this value. E.g. "3.354343" USDT
	/// ### Usage
	/// - show as formatted in toast in UI.
	pub required_allowance: String,
	/// Balance is formatted. E.g. "243.354343" USDT
	/// ### Usage
	/// - display in UI
	/// - compare with amount for err.
	pub balance: String,
	/// Est. fee is formatted. E.g. "1.23243" USDT
	/// ### Usage
	/// - display in UI
	pub est_fee: String,
}

/// Pre-OCP Values using NCW Params (for calc) for bulk pay.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PreOcpValuesNcwBulk {
	/// For each coin (unique)
	pub coin_entries: HashMap<StableCoin, PreOcpValuesNcwBulkCoin>,
	/// Total est. fee \
	/// Est. fee is formatted. E.g. "1.23243" USDT
	/// ### Usage
	/// - display in UI
	pub tot_est_fee: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PreOcpValuesNcwBulkCoin {
	/// Is coin allowance sufficient?
	/// NOTE: This field is redundant. But for direct use, added this field. Else, we have to
	/// convert required_allowance to U256 to check if it's zero.
	pub is_suff: bool,
	/// User need to approve this value. E.g. "3.354343" USDT
	/// ### Usage
	/// - show as formatted in toast in UI.
	pub required_allowance: String,
	/// Balance is formatted. E.g. "243.354343" USDT
	/// ### Usage
	/// - display in UI
	/// - compare with amount for err.
	pub balance: String,
}

/// Pre-OCP Values for NCW Params for single pay
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PreOcpValuesNcwParams {
	/// Allowance to `Permit2` need to use for comparo. So, "U256" in String.
	/// ### Usage
	/// - compare with amount for est. gas_usage
	pub allowance: String,
	/// Net balance is formatted.
	/// ### Usage
	/// - display in UI
	/// - compare with amount for err.
	pub balance: String,
	/// Gas price in wei.
	/// ### Usage
	/// - required in compute est. fee (in stablecoin).
	pub gas_price: u128,
	/// E.g. ETH, POL, .. in USD.
	/// ### Usage
	/// - required in compute est. fee (in stablecoin).
	pub gas_token_price: f64,
	/// E.g. USDT, USDC, .. in USD
	/// ### Usage
	/// - required in compute est. fee (in stablecoin).
	pub coin_price: f64,
}

/// Pre-OCP Values for NCW Params for bulk pay
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PreOcpValuesNcwParamsBulk {
	pub coin_entries: Vec<PreOcpValuesNcwParamsBulkCoin>,
	/// Gas price in wei.
	/// ### Usage
	/// - required in compute est. fee (in stablecoin).
	pub gas_price: u128,
	/// E.g. ETH, POL, .. in USD.
	/// ### Usage
	/// - required in compute est. fee (in stablecoin).
	pub gas_token_price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct PreOcpValuesNcwParamsBulkCoin {
	pub coin: StableCoin,
	/// Allowance to `Permit2` need to use for comparo. So, "U256" in String.
	/// ### Usage
	/// - compare with amount for est. gas_usage
	pub allowance: String,
	/// Net balance is formatted.
	/// ### Usage
	/// - display in UI
	/// - compare with amount for err.
	pub balance: String,
	/// E.g. USDT, USDC, .. in USD
	/// ### Usage
	/// - required in compute est. fee (in stablecoin).
	pub coin_price: f64,
}

#[derive(
	Archive,
	RkyvSerialize,
	RkyvDeserialize,
	Debug,
	Default,
	Serialize,
	Deserialize,
	Clone,
	PartialEq,
)]
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

impl PayOnchainPayload {
	/// sanitize address & amount
	///
	/// ## Notes
	/// - Here, individual santization applied for better error handling.
	///
	/// ## Returns
	/// - is_sanitized_full ✅
	/// - to_addr
	/// - amount
	#[cfg(feature = "utils")]
	pub fn is_sanitized(&self, use_in_ui: bool) -> (bool, Option<Address>, Option<U256>) {
		let to_addr = self.to_address.parse::<Address>().ok();
		let is_san_addr = to_addr.is_some();
		let parsed_amount = sanitize_and_parse_amount(&self.amount, self.coin, use_in_ui).ok();
		let is_san_amt = parsed_amount.is_some();

		(is_san_addr && is_san_amt, to_addr, parsed_amount)
	}
}

/// Currently, it's stored forever if repeat -> true, else when execute_at is done, we delete.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ScheduledPayOnchainPayload {
	// /// Sender's user_id
	// pub user_id: String,
	pub payload: PayOnchainPayload,
	/// Scheduled timestamp in secs.
	pub execute_at: i64,
	/// If `None`, then delete after `execute_at` elapsed during `scheduled_payments_handler` fn.
	pub repeat: Option<PaymentRecurrence>,
	/// Default: true. \
	/// ## Usage
	/// User can mark a (upcoming/due) scheduled payment as inactive so that the handler ignores
	/// it when running in background. This way, user doesn't have to delete the sched payment &
	/// can remain in DB until retention period (set to 30 days atm).
	pub is_active: bool,
}

impl Default for ScheduledPayOnchainPayload {
	fn default() -> Self {
		Self {
			payload: Default::default(),
			execute_at: Default::default(),
			repeat: Default::default(),
			is_active: true,
		}
	}
}

impl Display for ScheduledPayOnchainPayload {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl ScheduledPayOnchainPayload {
	/// set 1 min as min. scheduled time. \
	/// Set to 55 bcoz due to time in API server receiving req. from client.
	pub const MIN_SCHEDULED_DURATION: i64 = 55;
	pub const MAX_SCHEDULED_DURATION: i64 = 60 * 60 * 24 * 365; // 1 year

	/// sanitize s_payload & execute_time
	///
	/// ## Notes
	/// - Here, individual santization applied for better error handling.
	///
	/// ## Returns
	/// - is_sanitized_full ✅
	/// - address
	///   - if `addr.is_some()` -> sanitized.
	/// - amount
	///   - if `amount.is_some()` -> sanitized.
	/// - execute
	#[cfg(feature = "utils")]
	pub fn is_sanitized(&self, use_in_ui: bool) -> (bool, Option<Address>, Option<U256>, bool) {
		let (is_san_payload, to_addr, amount) = self.payload.is_sanitized(use_in_ui);

		let now = now_timestamp_secs() as i64;
		let is_san_execute = self.execute_at > now + Self::MIN_SCHEDULED_DURATION &&
			self.execute_at < now + Self::MAX_SCHEDULED_DURATION;

		(is_san_payload && is_san_execute, to_addr, amount, is_san_execute)
	}
}

pub trait CoinAmountLike {
	fn coin(&self) -> StableCoin;
	fn amount(&self) -> &str;
}

impl CoinAmountLike for ScheduledPayOnchainPayload {
	fn coin(&self) -> StableCoin {
		self.payload.coin
	}

	fn amount(&self) -> &str {
		&self.payload.amount
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PaymentRecurrence {
	Daily,
	Weekly,
	Monthly,
	Custom(i64),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GetScheduledPaymentsResponse {
	pub payments: Vec<ScheduledPayOnchainPayload>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub est_fee: Option<String>,
}

/* Platform */

/// ## Usage
/// - Only for logged-in devices
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Platform {
	TgBot,
	Web,
	CliMac,
	CliLinux,
	CliWindows,
	MacDesktop,
	LinuxDesktop,
	WindowsDesktop,
}

impl Display for Platform {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl From<Platform> for String {
	fn from(val: Platform) -> Self {
		val.to_string()
	}
}

impl Platform {
	pub const fn all() -> &'static [Platform] {
		use Platform as P;
		&[
			P::TgBot,
			P::Web,
			P::CliMac,
			P::CliLinux,
			P::CliWindows,
			P::MacDesktop,
			P::LinuxDesktop,
			P::WindowsDesktop,
		]
	}

	pub fn as_str(&self) -> &'static str {
		use Platform as P;
		match self {
			P::TgBot => "TgBot",
			P::Web => "Web",
			P::CliMac => "CliMac",
			P::CliLinux => "CliLinux",
			P::CliWindows => "CliWindows",
			P::MacDesktop => "MacDesktop",
			P::LinuxDesktop => "LinuxDesktop",
			P::WindowsDesktop => "WindowsDesktop",
		}
	}
}

impl FromStr for Platform {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"tgbot" => Ok(Platform::TgBot),
			"web" => Ok(Platform::Web),
			"climac" => Ok(Platform::CliMac),
			"clilinux" => Ok(Platform::CliLinux),
			"cliwindows" => Ok(Platform::CliWindows),
			"macdesktop" => Ok(Platform::MacDesktop),
			"linuxdesktop" => Ok(Platform::LinuxDesktop),
			"windowsdesktop" => Ok(Platform::WindowsDesktop),
			_ => Err("Invalid platform".to_string()),
		}
	}
}

/// this is
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NcJwtAuthPayload {
	pub message: String,
	pub signature: String,
	pub platform: Platform,
}

/// API
/// - Keys
/// - Metadata
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Api {
	pub metadata: ApiMetadata,
	pub keys: Vec<ApiKey>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ApiMetadata {
	pub plan: ApiPlan,
	pub duration: PaidPlanDuration,
	pub credits: i64,
	pub img_credits: i64,
	/// When the plan purchased or renewed, the expiry timestamp is set. For instance, plan is
	/// for a year. Then, expiry timestamp is set accordingly. By default, set for month for Free
	/// plan.
	pub expiry_at: i64,
}

impl Default for ApiMetadata {
	/// For FREE plan as default, we set to 1 month as expiry.
	fn default() -> Self {
		let plan = ApiPlan::Free;
		let duration = PaidPlanDuration::Month;
		Self {
			plan,
			duration,
			credits: plan.credit_quota(duration),
			img_credits: plan.img_credit_quota(duration),
			expiry_at: now_timestamp_secs() as i64 + duration.to_seconds(),
		}
	}
}

impl ApiMetadata {
	pub fn free() -> Self {
		Self::default()
	}

	/// Returns Ok(()) if the user is eligible to purchase a new API plan.
	///
	/// Eligibility:
	/// - current plan is free, or
	/// - current plan expired, or
	/// - credits are zero
	///
	/// ## Example
	/// ```rust,ignore
	/// let api_metadata = self.get_api_metadata(user_id).await?;
	/// api_metadata.is_eligible_for_purchase()?;
	/// ```
	pub fn is_eligible_for_purchase(&self) -> eyre::Result<()> {
		eyre::ensure!(
			self.plan.is_free() ||
				(now_timestamp_secs() as i64).ge(&self.expiry_at) ||
				self.credits.eq(&0),
			UfiError::InvalidCurrentPlanForApiPurchase
		);
		Ok(())
	}
}

/* API Key */

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct ApiKey {
	pub name: String,
	pub value: String,
	pub created_at: i64,
}

impl From<ApiKey> for Bson {
	fn from(api_key: ApiKey) -> Self {
		BsonDocument(doc! {
			"name": api_key.name,
			"value": api_key.value,
			"created_at": api_key.created_at,
		})
	}
}

#[derive(
	Archive, RkyvSerialize, RkyvDeserialize, Debug, Default, Serialize, Deserialize, PartialEq,
)]
pub struct Session {
	/// Address or Email
	pub user_id: String,
	pub jwt: String,
}

impl Session {
	pub fn new(user_id: &str, jwt: &str) -> Self {
		Self { user_id: user_id.to_owned(), jwt: jwt.to_owned() }
	}
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
pub struct ScheduledPaymentsFilter {
	/// - None: ignore
	/// - Some(chain): On chain
	#[serde(skip_serializing_if = "Option::is_none")]
	pub chain: Option<ChainName>,
	/// - None: ignore
	/// - Some(true): Upcoming => execute_at > now
	/// - Some(false): Due => execute_at <= now
	#[serde(skip_serializing_if = "Option::is_none")]
	pub timing: Option<bool>,
	/// - None: ignore
	/// - Some(true): Restrict past/due results to only those within `RETENTION_PERIOD`
	#[serde(skip_serializing_if = "Option::is_none")]
	pub upto_retention: Option<bool>,
	/// - None: ignore
	/// - Some(true): Active => is_active == true
	/// - Some(false): Inactive => is_active == false
	#[serde(skip_serializing_if = "Option::is_none")]
	pub activity: Option<bool>,
	/// Default: 60.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractPaymentsRequest {
	pub img_bytes: Vec<u8>,
	pub memo: Memo,
}
