use crate::{
	errors::UfiError,
	types::{
		ChainName, GasEstimate, PreOcpPayload, PreOcpValuesBulk, PreOcpValuesNcwAllowance,
		PreOcpValuesNcwBulk, PreOcpValuesNcwParams, PreOcpValuesNcwParamsBulk,
		PreOcpValuesNcwParamsBulkCoin, PreOcpValuesNcwSingleCoin, StableCoin,
	},
};
use alloy_primitives::{
	Address, U256,
	utils::{format_units, parse_units},
};
use eyre::{Context, OptionExt, ensure};
use std::{collections::HashMap, str::FromStr};

/// Format any num (in U256 String) to Decimal formatted considering coin's decimals.
pub fn fmt_value(num_in_u256_str: &str, coin: StableCoin) -> eyre::Result<String> {
	let value = format_units(U256::from_str(num_in_u256_str)?, coin.decimals())?;
	Ok(value)
}

pub fn is_value_gte(num_in_u256_str: &str, amount: &str, coin_decimals: u8) -> eyre::Result<bool> {
	Ok(num_in_u256_str.parse::<U256>()? >= parse_human_fmt_to_u256(amount, coin_decimals, true)?)
}

pub fn sanitize_address(address: &str) -> bool {
	address.parse::<Address>().is_ok()
}

/// Parse human readable format str to U256
///
/// User entering amount (in UI) is converted to U256 to `.to_string()` for sending via payload.
///
/// ## Example
/// input: "10.123456" \
/// output: "10123456"
///
/// ## Usage
/// - User entering amount as String compared to fetched net Onchain balance (U256) as validation.
///
/// NOTE: All tests inside Base layer.
///
/// ## Arguments
/// - use_in_ui:
///   - `false`: If values' decimals > coin_decimals, we skip error & instead curtail the decimals
///     to that of coin_decimals. E.g. value: `1.123456789012345678` => `1.123456` => `1123456` (in
///     U256).
///   - `true`: Usage inside web app, to show err when value's decimals > coin_decimals.
pub fn parse_human_fmt_to_u256(
	value: &str,
	coin_decimals: u8,
	use_in_ui: bool,
) -> eyre::Result<U256> {
	// NOTE: this line added due to a small case where in `pending_amount` field in DB is set to
	// "0E-18" instead of "0.00000000...000" for DAI. It was found that this was done inside
	// `execute_bundle` fn during u128 arithmetic at mongoDB level that can't be controlled from
	// code here. So, covered the case. Else, there would be invalid digit error in case of DAI or
	// any token with such big decimals (7-18). Didn't notice this issue in case of 6 decimals
	// tokens like USDT, USDC.
	let num = if value.starts_with("0E-") { "0.0" } else { value };

	// Split the input into whole and fractional parts
	let parts: Vec<&str> = num.split('.').collect();

	// Parse and scale the fractional part
	let fractional_part = if let Some(frac) = parts.get(1) {
		if use_in_ui {
			ensure!(
				frac.len() <= coin_decimals as usize,
				UfiError::MaxDecimalsReached(coin_decimals)
			);
		}
		let scale = 10u128.pow(frac.len() as u32);
		let frac_num = frac.parse::<u128>().unwrap_or(0); // Parse fractional digits as integer
		U256::from(frac_num) * U256::from(10u128.pow(coin_decimals as u32)) / U256::from(scale)
	} else {
		U256::ZERO
	};

	// Parse the whole part
	let whole_part =
		if let Some(whole) = parts.first() { whole.parse::<U256>()? } else { U256::ZERO };

	// Scale the whole part and combine with fractional part
	let scaled_whole_part = whole_part * U256::from(10u128.pow(coin_decimals as u32));
	Ok(scaled_whole_part + fractional_part)
}

/// Resolve NCW Gas usage and allowance.
///
/// NOTE: 50k gas additionally added for the fee payment to admin.
/// Why 50k? 👉 Actual spent is around 112k gas, which might increase.
/// So, for safety 170k (120 + 50) gas as fee -> Admin. Now, when the act. amount is 112k =>
/// refund: 170k - 112k = 58k gas ~ $ (price @ at that time).
///
/// ## Arguments
/// - ..
/// - `is_single_or_bulkadminfee`:
///   - true: single or, gas usage for fee pay to admin.
///   - false: bulk. Here, `permit_transfer_from` could be multiplied with pay_count for that coin
///     in a batch.
fn resolve_ncw_gas_usage_and_allowance(
	is_suff: bool,
	allowance: U256,
	target_amt: U256,
	approve: u128,
	permit_transfer_gas: u128,
	is_single_or_bulkadminfee: bool,
) -> (u128, U256) {
	let extra = if is_single_or_bulkadminfee { 50_000 } else { 0 };
	if is_suff {
		(permit_transfer_gas + extra, U256::ZERO)
	} else {
		(approve + permit_transfer_gas + extra, target_amt.saturating_sub(allowance))
	}
}

/// Normalize a value to higher decimals before summing mixed-decimal fees. \
/// E.g. 6 -> 18.
///
/// ## Example
/// ```text
/// Mixed estimated fees before normalization:
/// - USDT (6 dp):  1.232340              -> 1232340
/// - DAI  (18 dp): 2.232432323200000000  -> 2232432323200000000
/// - USDT (6 dp): 11.232434              -> 11232434
/// - USDC (6 dp):  3.343535              -> 3343535
///
/// These raw U256 values must NOT be summed directly,
/// because they are expressed using different decimal precisions.
///
/// So, first normalize all values to the max decimals in the batch
/// (here: 18 decimals):
///
/// - 1232340             -> 1232340000000000000
/// - 2232432323200000000 -> 2232432323200000000
/// - 11232434            -> 11232434000000000000
/// - 3343535             -> 3343535000000000000
///
/// Now all values are in the same decimal space,
/// so summation becomes correct:
///
///   1232340000000000000
/// + 2232432323200000000
/// +11232434000000000000
/// + 3343535000000000000
/// =18123941323200000000
///
/// Human-readable (18 decimals):
/// 18.123941323200000000
/// ```
///
/// ## Usage
/// - In a mix of amounts with varying decimals, all values should be normalized before adding like
///   in `tot_est_fee` in bulk pay.
fn normalize_est_fee_u256(
	est_fee_u256: U256,
	from_decimals: u8,
	to_decimals: u8,
	context: &str,
) -> eyre::Result<U256> {
	let val = if from_decimals == to_decimals {
		est_fee_u256
	} else {
		let scale = U256::from(10).pow(U256::from((to_decimals - from_decimals) as u32));
		est_fee_u256
			.checked_mul(scale)
			.ok_or_eyre(format!("Overflow while normalizing {context}"))?
	};

	Ok(val)
}

/// Convert a normalized value (common decimal space) back to a coin's native decimals. \
/// E.g. 18 -> 6.
///
/// ## Example
/// ```text
/// Suppose total estimated fee was computed in normalized 18 decimals:
///
/// 18123941323200000000  (18 dp)
///
/// For a 6-decimal coin (e.g. USDT/USDC), convert back:
///
/// 18123941323200000000 -> 18123941
///
/// Human-readable:
/// 18.123941
/// ```
///
/// ## Why this is required
/// - All mixed-decimal values are first normalized to a common decimal space (e.g. 18) for safe
///   summation.
/// - Before returning or displaying, values must be converted back to the target coin's native
///   decimals.
///
/// ## Usage
/// - Used after summing normalized fees (e.g. `total_est_fee_u256` in bulk pay) to convert the
///   result into the fee coin's decimals before formatting.
fn denormalize_est_fee_u256(
	est_fee_u256: U256,
	from_decimals: u8,
	to_decimals: u8,
) -> eyre::Result<U256> {
	if from_decimals == to_decimals {
		Ok(est_fee_u256)
	} else if from_decimals > to_decimals {
		let scale = U256::from(10).pow(U256::from((from_decimals - to_decimals) as u32));
		Ok(est_fee_u256 / scale)
	} else {
		let scale = U256::from(10).pow(U256::from((to_decimals - from_decimals) as u32));
		est_fee_u256
			.checked_mul(scale)
			.ok_or_eyre("Overflow while denormalizing estimated fee")
	}
}

#[allow(clippy::too_many_arguments)]
fn calc_ncw_est_fee_snapshot(
	allowance: U256,
	target_amt: U256,
	approve: u128,
	permit_transfer_gas: u128,
	is_single_or_bulkadminfee: bool,
	gas_price: u128,
	gas_token_price: f64,
	platform_payment_fee_multiplier: f64,
	price_denom: f64,
	coin_decimals: u8,
) -> eyre::Result<(U256, U256)> {
	// NOTE: For C Pay, allowance is either 0 or MAX unlike in NC Pay.
	// For NC Pay, avoid treating a zero target amount as allowance-sufficient just because
	// allowance is non-zero. Empty/zero amount is only used for initial UI fee estimation,
	// and marking it sufficient would incorrectly select the cheaper no-approve gas path.
	let is_suff = !target_amt.is_zero() && !allowance.is_zero() && allowance.ge(&target_amt);

	let (est_gas_usage, required_allowance_val) = resolve_ncw_gas_usage_and_allowance(
		is_suff,
		allowance,
		target_amt,
		approve,
		permit_transfer_gas,
		is_single_or_bulkadminfee,
	);

	// fee = (gas_usage * gas_price * gas_token_price * platform_multiplier)
	//       / price_denom
	let est_gas_fee = (est_gas_usage * gas_price) as f64;
	let est_fee_f64 = est_gas_fee * gas_token_price * platform_payment_fee_multiplier / price_denom;
	let est_fee_u256 = parse_human_fmt_to_u256(&est_fee_f64.to_string(), coin_decimals, false)?;

	Ok((est_fee_u256, required_allowance_val))
}

fn get_required_allowance(allowance: U256, target_amt: U256) -> U256 {
	// NOTE: For C Pay, allowance is either 0 or MAX unlike in NC Pay.
	let is_suff = !allowance.is_zero() && allowance.ge(&target_amt);

	if is_suff { U256::ZERO } else { target_amt - allowance }
}

const PLATFORM_PAYMENT_FEE_BPS: u128 = 11_500;
const PLATFORM_PAYMENT_FEE_MULTIPLIER: f64 = PLATFORM_PAYMENT_FEE_BPS as f64 / 10_000.0;

/// Strict, checked parsing for payment drafts. Never truncate fractional digits or
/// accept malformed fractional text while deciding whether a payment is affordable.
pub fn parse_payment_amount(value: &str, coin: StableCoin) -> eyre::Result<U256> {
	let value = value.trim();
	ensure!(!value.is_empty(), "Invalid amount");
	let mut parts = value.split('.');
	let whole = parts.next().unwrap_or_default();
	let fractional = parts.next().unwrap_or_default();
	ensure!(
		parts.next().is_none() &&
			(!whole.is_empty() || !fractional.is_empty()) &&
			whole.bytes().all(|b| b.is_ascii_digit()) &&
			fractional.bytes().all(|b| b.is_ascii_digit()),
		"Invalid amount"
	);
	let decimals = coin.decimals();
	ensure!(fractional.len() <= decimals as usize, UfiError::MaxDecimalsReached(decimals));
	let scale = U256::from(10).pow(U256::from(decimals));
	let whole = if whole.is_empty() { U256::ZERO } else { U256::from_str(whole)? };
	let whole = whole.checked_mul(scale).ok_or_eyre("Amount is too large")?;
	let fraction = if fractional.is_empty() {
		U256::ZERO
	} else {
		U256::from_str(fractional)?
			.checked_mul(U256::from(10).pow(U256::from(decimals as usize - fractional.len())))
			.ok_or_eyre("Amount is too large")?
	};
	whole.checked_add(fraction).ok_or_eyre("Amount is too large")
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaymentCoinQuote {
	pub coin: StableCoin,
	pub amount: String,
	pub balance: String,
	pub est_fee: String,
	pub shortfall: Option<String>,
	/// Positions in the original recipient list whose positive amounts exceed this coin's
	/// balance when allocated from top to bottom, after reserving its batch fee.
	pub insufficient_recipients: Vec<usize>,
}

/// The wallet-specific prefetch responses used to validate a payment draft.
pub enum PaymentBatchParams<'a> {
	Custodial { chain: ChainName, params: &'a PreOcpValuesBulk },
	NonCustodial { chain: ChainName, params: &'a PreOcpValuesNcwParamsBulk },
}

/// Aggregate selected coins and validate totals plus fees against live balances.
/// Both wallet modes charge one batch estimate in the last item's coin.
pub fn payment_batch_quote(
	params: PaymentBatchParams<'_>,
	items: &[(StableCoin, String)],
) -> eyre::Result<Vec<PaymentCoinQuote>> {
	let fee_coin = items.last().ok_or_eyre("No recipients")?.0;
	let mut totals = HashMap::<StableCoin, U256>::new();
	let mut counts = HashMap::<StableCoin, u32>::new();
	let mut parsed_items = Vec::with_capacity(items.len());
	for (coin, amount) in items {
		let amount = if amount.trim().is_empty() {
			U256::ZERO
		} else {
			parse_payment_amount(amount, *coin)?
		};
		let total = totals.entry(*coin).or_default();
		*total = total.checked_add(amount).ok_or_eyre("Total amount is too large")?;
		parsed_items.push((*coin, amount));
		let count = counts.entry(*coin).or_default();
		*count = count.checked_add(1).ok_or_eyre("Too many recipients")?;
	}
	let selected_coins = StableCoin::all().iter().copied().filter(|coin| totals.contains_key(coin));
	let inputs = match params {
		PaymentBatchParams::Custodial { chain, params } => {
			let fee = parse_payment_amount(
				&compute_est_fee_cw_bulkpay(chain, fee_coin, &counts, params)?,
				fee_coin,
			)?;
			selected_coins
				.map(|coin| {
					let entry = params
						.coin_entries
						.iter()
						.find(|entry| entry.coin == coin)
						.ok_or_eyre(format!("Balance unavailable for {coin}"))?;
					Ok((
						coin,
						entry.balance.as_str(),
						if coin == fee_coin { fee } else { U256::ZERO },
					))
				})
				.collect::<eyre::Result<Vec<_>>>()?
		},
		PaymentBatchParams::NonCustodial { chain, params } => {
			ensure!(
				params.gas_token_price.is_finite() && params.gas_token_price > 0.0,
				"Invalid gas price inputs"
			);
			let selected = selected_coins
				.map(|coin| {
					let entry = params
						.coin_entries
						.iter()
						.find(|entry| entry.coin == coin)
						.ok_or_eyre(format!("Balance unavailable for {coin}"))?;
					ensure!(
						entry.coin_price.is_finite() && entry.coin_price > 0.0,
						"Invalid coin price"
					);
					Ok(entry)
				})
				.collect::<eyre::Result<Vec<_>>>()?;
			let bulk_fee = compute_est_fee_ncw_bulkpay(
				chain,
				fee_coin,
				totals.clone(),
				counts,
				PreOcpValuesNcwParamsBulk {
					coin_entries: selected.iter().map(|entry| (*entry).clone()).collect(),
					gas_price: params.gas_price,
					gas_token_price: params.gas_token_price,
					..Default::default()
				},
				false,
				true,
			)?
			.tot_est_fee;
			selected
				.into_iter()
				.map(|entry| {
					let fee = if entry.coin == fee_coin {
						parse_payment_amount(&bulk_fee, entry.coin)?
					} else {
						U256::ZERO
					};
					Ok((entry.coin, entry.balance.as_str(), fee))
				})
				.collect::<eyre::Result<Vec<_>>>()?
		},
	};
	inputs
		.into_iter()
		.map(|(coin, balance, fee)| {
			let amount = totals[&coin];
			let parsed_balance = parse_payment_amount(balance, coin)?;
			let required = amount.checked_add(fee).ok_or_eyre("Total plus fee is too large")?;
			let spendable = parsed_balance.saturating_sub(fee);
			let mut cumulative = U256::ZERO;
			let mut insufficient_recipients = Vec::new();
			for (index, (recipient_coin, recipient_amount)) in parsed_items.iter().enumerate() {
				if *recipient_coin != coin {
					continue;
				}
				cumulative = cumulative
					.checked_add(*recipient_amount)
					.ok_or_eyre("Total amount is too large")?;
				if !recipient_amount.is_zero() && cumulative > spendable {
					insufficient_recipients.push(index);
				}
			}
			let shortfall = required
				.checked_sub(parsed_balance)
				.filter(|value| !value.is_zero())
				.map(|value| fmt_output(value, coin.decimals()))
				.transpose()?;
			Ok(PaymentCoinQuote {
				coin,
				amount: fmt_output(amount, coin.decimals())?,
				balance: balance.to_owned(),
				est_fee: fmt_output(fee, coin.decimals())?,
				shortfall,
				insufficient_recipients,
			})
		})
		.collect()
}

/// Calculate one custodial batch estimate from live prefetched metadata.
/// The custodial execution policy uses MAX BundlePay approvals: a zero allowance adds
/// funding and approval gas once per selected coin; recipient transfers scale by count.
/// No Permit2, signing, or extra NCW fee-transfer gas is included.
pub fn compute_est_fee_cw_bulkpay(
	chain: ChainName,
	fee_coin: StableCoin,
	payments_per_coin: &HashMap<StableCoin, u32>,
	params: &PreOcpValuesBulk,
) -> eyre::Result<String> {
	ensure!(!payments_per_coin.is_empty(), "No recipients");
	ensure!(payments_per_coin.contains_key(&fee_coin), "Fee coin is not selected");
	let usd_units = |price: f64| -> eyre::Result<U256> {
		ensure!(price.is_finite() && price > 0.0, "Invalid coin price");
		let units = parse_payment_amount(&format!("{price:.18}"), StableCoin::DAI)?;
		ensure!(!units.is_zero(), "Coin price is below supported precision");
		Ok(units)
	};
	let gas_token_price = usd_units(params.gas_token_price)?;
	let fee_coin_entry = params
		.coin_entries
		.iter()
		.find(|entry| entry.coin == fee_coin)
		.ok_or_eyre("Missing fee coin")?;
	let fee_coin_price = usd_units(fee_coin_entry.coin_price)?;
	let mut total_gas = 0_u128;
	for (coin, count) in payments_per_coin {
		ensure!(*count > 0, "Recipient count must be positive");
		let entry = params
			.coin_entries
			.iter()
			.find(|entry| entry.coin == *coin)
			.ok_or_eyre(format!("Balance unavailable for {coin}"))?;
		let allowance = U256::from_str(&entry.allowance).wrap_err("Invalid BundlePay allowance")?;
		let GasEstimate { eth_transfer, approve, transfer_from, .. } =
			chain.get_gas_usage_limit(Some(*coin));
		let transfer_gas = transfer_from
			.checked_mul(u128::from(*count))
			.ok_or_eyre("Transfer gas is too large")?;
		let approval_gas = if allowance.is_zero() {
			eth_transfer.checked_add(approve).ok_or_eyre("Approval gas is too large")?
		} else {
			0
		};
		total_gas = total_gas
			.checked_add(transfer_gas)
			.and_then(|gas| gas.checked_add(approval_gas))
			.ok_or_eyre("Batch gas is too large")?;
	}
	let gas_wei = total_gas.checked_mul(params.gas_price).ok_or_eyre("Gas fee is too large")?;
	// Prices share an 18-decimal USD scale, which cancels in their ratio. Use checked
	// integer arithmetic and round up once, after aggregating the whole batch.
	let numerator = U256::from(gas_wei)
		.checked_mul(gas_token_price)
		.and_then(|value| value.checked_mul(U256::from(PLATFORM_PAYMENT_FEE_BPS)))
		.and_then(|value| value.checked_mul(U256::from(10).pow(U256::from(fee_coin.decimals()))))
		.ok_or_eyre("Batch fee is too large")?;
	let denominator = fee_coin_price
		.checked_mul(U256::from(10).pow(U256::from(chain.to_gas_coin().decimals())))
		.and_then(|value| value.checked_mul(U256::from(10_000)))
		.ok_or_eyre("Fee price denominator is too large")?;
	let fee = (numerator / denominator)
		.checked_add(U256::from(u8::from(!(numerator % denominator).is_zero())))
		.ok_or_eyre("Batch fee is too large")?;
	fmt_output(fee, fee_coin.decimals())
}

/// Compute Est. fee for NC Pay.
///
///
/// ## Notes
/// - This fn should't be `async` bcoz on change of amount on UI, the text box err (if any) should
///   show synchronously on every digit modification in amount.
/// - If the fee is:
///   - excl:
///     - The user-entered `amount` does NOT include fees.
///     - An initial `est_fee` is computed based only on the entered amount (or even empty/0).
///     - Then, the effective spend becomes:
///
///           total_spend = amount + est_fee
///
///     - This `total_spend` is checked against allowance to determine whether approval is required.
///     - If approval is required, gas usage increases → which increases `est_fee`.
///
///     - Hence, fee estimation indirectly depends on allowance, and allowance depends on
///       `total_spend`, which itself depends on `est_fee`.
///
///     - This introduces a dependency loop:
///
///           est_fee → total_spend → allowance_check → gas_path → est_fee
///
///     - Therefore, a re-computation step is required to stabilize the final fee.
///
///   - incl:
///     - The user-entered `amount` already includes fees.
///     - Hence, `amount` directly represents the total spend:
///
///           total_spend = amount
///
///     - From this, we derive: payee_amount = amount - est_fee admin_fee    = est_fee
///
///     - Since `total_spend` is fixed upfront, the allowance check is stable:
///         - allowance >= total_spend → no approve needed
///         - allowance < total_spend  → approve required
///
///     - The gas path (approve vs no approve) is determined in a single pass, so `est_fee` does not
///       require re-computation.
///
///     - Therefore, unlike `fee_excl`, there is no dependency loop and a single calculation is
///       sufficient.
///
/// ## Arguments
/// - `payload`: selected {coin, chain}
/// - `amt_or_tot_amount`:
///   - Represents the **input value to be parsed into `tot_amount`** for fee + allowance logic.
///
///   - Interpretation depends on `is_fee_incl`:
///
///     - fee_excl:
///       - Input is the **user-entered amount only** (fees NOT included).
///       - Parsed as:
///
///             tot_amount = amount
///
///       - Final spend becomes `amount + est_fee` after re-computation.
///
///     - fee_incl:
///       - Input is the **total spend (amount already includes fee)**.
///       - Parsed directly as:
///
///             tot_amount = amount  // already includes fee
///
///       - No adjustment needed; this value is treated as final spend.
///
///   - Example:
///       - fee_excl: "10.12"  → means payee gets 10.12, fee added later
///       - fee_incl: "10.12"  → means total spend is 10.12 (payee gets amount - fee)
/// - `pre_ocp_values`: params (from fn: `prefetch_ncw_balance_fee_params`) for calculating est fees
///   synchronously.
/// - `is_fee_incl`
/// - `use_is_ui`: When used in UI, we skip the `InsufficientBalance` check bcoz we don't want the
///   UI to collapse showing Error card. But, if you are running an example program, then you should
///   set to `false`, bcoz then the code returns early as `InsufficientBalance`.
///
/// There are 2 pages: ApiPlan, FliQPay, where we might set it to false, as we would want the
/// "InsufficientBalance" to be checked before making the payment. Actually, in these pages, we
/// don't get to edit the amount, so need to check for it. It is checked once via
/// `validate_and_parse_amount` fn.
///
/// ## Returns
/// - `is_coin_allowance_suff`: NOTE: due to this field returned, we don't have to convert the
///   formatted required allowance to U256 for zero check as `value.is_zero()`.
///   - true  → sufficient, no approval needed.
///   - false → insufficient, prompt: "Please approve X amount".
/// - required_allowance: X value is to be approved by payer to Permit2. E.g. `21.34545` USDT or
///   "0.00" USDT.
/// - is MAX allowance
/// - formatted est fees. E.g. `0.132433` USDT or "0.00" USDT.
pub fn compute_est_fee_ncw(
	payload: PreOcpPayload,
	amt_or_tot_amount: &str,
	pre_ocp_values: &PreOcpValuesNcwParams,
	is_fee_incl: bool,
	use_in_ui: bool,
) -> eyre::Result<(bool, String, bool, String)> {
	// 1. Destructure and Parse Inputs immediately
	let PreOcpPayload { coin, chain } = payload;
	let PreOcpValuesNcwParams {
		allowance: allowance_str,
		balance: balance_str,
		gas_price,
		gas_token_price,
		coin_price,
		..
	} = pre_ocp_values;

	let coin_decimals = coin.decimals();
	let allowance = U256::from_str(allowance_str).wrap_err("Failed to parse allowance")?;
	let mut tot_amount = parse_human_fmt_to_u256(amt_or_tot_amount, coin_decimals, false)
		.wrap_err("Failed to parse amount")?;

	// NOTE: Disabled because the Pay UI should display estimated fees before the user
	// enters an amount (e.g., initial render / empty input state).
	// ensure!(!tot_amount.is_zero(), "Amount must be non-zero.");

	let balance = parse_human_fmt_to_u256(balance_str, coin_decimals, false)?;

	// 2. Early Balance Check (Fail fast)
	if !use_in_ui && tot_amount.gt(&balance) {
		return Err(UfiError::InsufficientBalance.into())
	}

	// 3. Pre-calculate Constants
	let GasEstimate { approve, permit_transfer_from, .. } = chain.get_gas_usage_limit(None);

	// Optimization: Calculate price denominator once.
	// Formula: coin_price * 10^(gas_coin_decimals)
	let gas_coin_decimals = chain.to_gas_coin().decimals() as i32;
	let price_denom = coin_price * 10f64.powi(gas_coin_decimals);

	// 4. Define Calculation Logic (Closure to handle repetition)
	// Returns: (Calculated Fee U256, Required allowance val)
	// NOTE: Add platform_fee on top of network fee.

	// 5. Initial Calculation
	let (mut est_fee_u256, mut required_allowance_val) = calc_ncw_est_fee_snapshot(
		allowance,
		tot_amount,
		approve,
		permit_transfer_from,
		true,
		*gas_price,
		*gas_token_price,
		PLATFORM_PAYMENT_FEE_MULTIPLIER,
		price_denom,
		coin_decimals,
	)?;

	// 6. Conditional Re-calculation (Fee-Excl. Case)
	//
	// In `fee_excl` mode, the user-entered `amount` does NOT include fees.
	// However, the actual spend becomes:
	//
	//     total_spend = amount + est_fee
	//
	// This introduces a dependency loop:
	//
	//     est_fee → total_spend → allowance_check → gas_path → est_fee
	//
	// Why recomputation is required:
	// - The initial `est_fee` is calculated using only `amount`.
	// - Once we add `est_fee` to form `total_spend`, the allowance condition may change:
	//     - allowance >= amount      → no approve needed
	//     - allowance < total_spend  → approve required
	//
	// - This change affects the gas path:
	//     - permit_transfer_from           (no approve)
	//     - approve + permit_transfer_from (with approve)
	//
	// - Since gas usage changes, the fee also changes.
	//
	// Therefore:
	// 1. Add `est_fee` to `amount` to get the real spend (`total_spend`).
	// 2. Re-run the fee + allowance computation using this updated value.
	//
	// NOTE:
	// - This is a 2-pass approximation that is sufficient in practice.
	// - Further iterations are unnecessary as the system stabilizes quickly.
	// - For `fee_incl`, this step is skipped because `amount` already represents total spend.
	if !is_fee_incl {
		tot_amount = tot_amount
			.checked_add(est_fee_u256)
			.ok_or_eyre("Amount plus fee is too large")?;
		if !use_in_ui && tot_amount.gt(&balance) {
			return Err(UfiError::InsufficientBalance.into())
		}

		// Check if adding the fee pushed us over the allowance threshold
		(est_fee_u256, required_allowance_val) = calc_ncw_est_fee_snapshot(
			allowance,
			tot_amount,
			approve,
			permit_transfer_from,
			true,
			*gas_price,
			*gas_token_price,
			PLATFORM_PAYMENT_FEE_MULTIPLIER,
			price_denom,
			coin_decimals,
		)?;
	}

	// 7. Format Output
	let is_suff = required_allowance_val.is_zero();
	let required_allowance_val_fmt = fmt_output(required_allowance_val, coin_decimals)?;
	let est_fee_fmt = fmt_output(est_fee_u256, coin_decimals)?;

	Ok((is_suff, required_allowance_val_fmt, allowance == U256::MAX, est_fee_fmt))
}

/// Compute estimated fee for NC **Bulk Pay** across multiple coins.
///
/// This function extends `compute_est_fee_ncw` (single-pay) to handle a batch of payments
/// involving **multiple coins**, where each coin may have a different allowance state,
/// balance, and gas path.
///
/// ## Key Properties
/// - **Per-coin allowance evaluation**: Each coin is evaluated independently for allowance
///   sufficiency:
///
///   allowance >= total_spend_for_that_coin
///
///   This determines whether the gas pathincludes `approve` or not.
///
/// - **Correct gas path per coin**: Gas usage differs per coin depending on allowance:
///   - sufficient allowance → `permit_transfer_from`
///   - insufficient allowance → `approve + permit_transfer_from`
///
/// - **2-pass recomputation (fee-excl only)**: Same logic as single-pay: ```text est_fee →
///   total_spend → allowance_check → gas_path → est_fee ``` Hence:
///   1. Compute initial fee using raw amount.
///   2. Add fee to amount (`fee_excl`).
///   3. Recompute fee using updated total.
///
///   This ensures correct handling when allowance sufficiency flips after including fee.
///
/// - **No naive multiplication**: Unlike older logic (`est_fee * batch_len`), this function:
///   - computes fee per coin based on its actual gas path
///   - sums all per-coin fees
///
///   This is critical because:
///   - some coins may require approval
///   - others may not
///   - resulting in different gas usage per coin
///
/// ## Arguments
/// - `chain`: selected chain
/// - `tot_amount_per_coin`: Aggregated total amount per coin across the batch
///
///   USDT → sum(all USDT entries)
///   USDC → sum(all USDC entries)
///
/// - `params`: pre-fetched values from `prefetch_ncw_balance_fee_params_bulkpay`
/// - `is_fee_incl`: whether amount already includes fee
/// - `use_in_ui`: skips hard balance validation when true
///
/// ## Returns
/// - `PreOcpValuesNcwBulk`
///   - `coin_entries`:
///     - `is_suff`: allowance sufficiency per coin
///     - `required_allowance`: required approval amount per coin
///     - `balance`: formatted balance
///   - `est_fee`:
///     - total estimated fee = **sum of per-coin computed fees**
///     - NOT derived from `batch_len`
///
/// ## Important Note
/// Every transfer's gas cost is converted using the selected fee coin's price and
/// decimals. The entire estimate is charged once in that coin. If adding the fee
/// crosses its allowance, the estimate also includes that coin's approval cost.
///
/// ## Design pros
/// - Handles mixed allowance states across coins
/// - Avoids incorrect fee scaling via `batch_len`. E.g. est_fee * batch_len
/// - Mirrors single-pay correctness guarantees
/// - Deterministic and stable after 2-pass computation (in case of fee-excl.)
///
/// TODO: est_fee should 📉 as batch size 📈.
/// Test fn: `_001_test_estfee_scales_as_batch_size_increase`
/// WARN: Currently, it's increasing.
pub fn compute_est_fee_ncw_bulkpay(
	chain: ChainName,
	fee_coin: StableCoin,
	tot_amount_per_coin: HashMap<StableCoin, U256>,
	payments_per_coin: HashMap<StableCoin, u32>,
	params: PreOcpValuesNcwParamsBulk,
	is_fee_incl: bool,
	use_in_ui: bool,
) -> eyre::Result<PreOcpValuesNcwBulk> {
	let PreOcpValuesNcwParamsBulk {
		coin_entries: params_coin_entries,
		gas_price,
		gas_token_price,
		..
	} = params;

	ensure!(
		params_coin_entries.len().le(&StableCoin::all().len()) &&
			tot_amount_per_coin.len().le(&StableCoin::all().len()),
		"Params coin entries & tot amount per coin can't exceed coins max. length"
	);

	let fee_coin_price = params_coin_entries
		.iter()
		.find(|entry| entry.coin == fee_coin)
		.ok_or_eyre("Missing fee coin")?
		.coin_price;
	ensure!(fee_coin_price.is_finite() && fee_coin_price > 0.0, "Invalid fee coin price");
	let fee_coin_decimals = fee_coin.decimals();
	let max_decimals = params_coin_entries
		.iter()
		.map(|entry| entry.coin.decimals())
		.max()
		.unwrap_or(StableCoin::default().decimals());

	let mut coin_entries = HashMap::with_capacity(params_coin_entries.len());
	let mut total_est_fee_u256 = U256::ZERO;
	let gas_coin_decimals = chain.to_gas_coin().decimals() as i32;

	let mut fee_coin_balance = U256::ZERO;
	let mut fee_coin_allowance = U256::ZERO;
	let mut fee_coin_totamount = U256::ZERO;

	for entry in params_coin_entries.iter() {
		let PreOcpValuesNcwParamsBulkCoin { coin, allowance, balance, .. } = entry;
		let coin = *coin;
		let coin_decimals = coin.decimals();

		let allowance = U256::from_str(allowance).wrap_err("Failed to parse allowance")?;
		let balance = parse_human_fmt_to_u256(balance, coin_decimals, false)
			.wrap_err("Failed to parse balance")?;

		let tot_amount = tot_amount_per_coin
			.get(&coin)
			.copied()
			.ok_or_eyre(format!("Missing total amount for coin: {coin:?}"))?;

		if !use_in_ui && tot_amount.gt(&balance) {
			return Err(UfiError::InsufficientBalance.into())
		}

		let GasEstimate { approve, permit_transfer_from, .. } = chain.get_gas_usage_limit(None);

		// fee = (gas_usage * gas_price * gas_token_price * platform_multiplier)
		//       / (coin_price * 10^(gas_coin_decimals))
		// All gas costs are paid in the fee coin, including transfers of other coins.
		let price_denom = fee_coin_price * 10f64.powi(gas_coin_decimals);

		let payment_count = payments_per_coin
			.get(&coin)
			.ok_or_eyre(format!("Failed to get pay count for {coin}"))?
			.to_owned();
		// NOTE:
		// - In a bulk batch, only **one `approve`** is required per coin (if allowance is
		//   insufficient).
		// - However, `permit_transfer_from` is executed **per payment**, so its gas cost scales
		//   with the number of payments for that coin.
		//
		// Hence:
		//     total_permit_transfer_gas = permit_transfer_from * payment_count
		let mut total_permit_transfer_gas = permit_transfer_from * payment_count as u128;
		if fee_coin == coin {
			// NOTE: add 50k gas extra based on fee payment to admin.
			total_permit_transfer_gas += 50_000;

			fee_coin_balance = balance;
			fee_coin_allowance = allowance;
			fee_coin_totamount = tot_amount;
		}
		let (est_fee_u256, required_allowance_val) = calc_ncw_est_fee_snapshot(
			allowance,
			tot_amount,
			approve,
			total_permit_transfer_gas,
			false,
			gas_price,
			gas_token_price,
			PLATFORM_PAYMENT_FEE_MULTIPLIER,
			price_denom,
			fee_coin_decimals,
		)?;

		// est_fee directly not added as there could be mixed decimals related issue. So, we
		// normalize by scaling to 10^remaining (e.g. 18-6 = 12), if required.
		let normalized_est_fee_u256 = normalize_est_fee_u256(
			est_fee_u256,
			fee_coin_decimals,
			max_decimals,
			"bulk-pay estimated fee",
		)?;

		total_est_fee_u256 = total_est_fee_u256
			.checked_add(normalized_est_fee_u256)
			.ok_or_eyre("Overflow while summing bulk-pay estimated fees")?;

		coin_entries.insert(
			coin,
			PreOcpValuesNcwSingleCoin {
				allowance: PreOcpValuesNcwAllowance {
					is_suff: required_allowance_val.is_zero(),
					required_allowance: fmt_output(required_allowance_val, coin_decimals)?,
					is_max_allowance: allowance == U256::MAX,
				},
				balance: entry.balance.to_owned(),
			},
		);
	}

	// Same reasoning as single-pay, but only do this for `fee_coin == coin`.
	// for fee-excl, total spend becomes (fee_coin_totamount + tot_est_fee), which can change
	// allowance sufficiency, so recompute using the updated spend i.e. `fee_coin_totamount`.
	if !is_fee_incl {
		// denormalize total_est_fee_u256 before adding.
		let mut fee =
			denormalize_est_fee_u256(total_est_fee_u256, max_decimals, fee_coin_decimals)?;
		let initial_amount = fee_coin_totamount;
		fee_coin_totamount =
			initial_amount.checked_add(fee).ok_or_eyre("Amount plus fee is too large")?;
		// Adding the batch fee can cross an otherwise sufficient allowance. Include
		// the newly required approval cost before checking spendable balance.
		if !initial_amount.is_zero() &&
			!fee_coin_allowance.is_zero() &&
			fee_coin_allowance >= initial_amount &&
			fee_coin_allowance < fee_coin_totamount
		{
			let (extra_fee, _) = calc_ncw_est_fee_snapshot(
				U256::ZERO,
				U256::from(1),
				chain.get_gas_usage_limit(None).approve,
				0,
				false,
				gas_price,
				gas_token_price,
				PLATFORM_PAYMENT_FEE_MULTIPLIER,
				fee_coin_price * 10f64.powi(gas_coin_decimals),
				fee_coin_decimals,
			)?;
			fee = fee.checked_add(extra_fee).ok_or_eyre("Fee is too large")?;
			total_est_fee_u256 = normalize_est_fee_u256(
				fee,
				fee_coin_decimals,
				max_decimals,
				"bulk fee with approval",
			)?;
			fee_coin_totamount =
				initial_amount.checked_add(fee).ok_or_eyre("Amount plus fee is too large")?;
		}

		if !use_in_ui && fee_coin_totamount.gt(&fee_coin_balance) {
			return Err(UfiError::InsufficientBalance.into())
		}

		let required_allowance_val = get_required_allowance(fee_coin_allowance, fee_coin_totamount);
		let required_allowance = fmt_output(required_allowance_val, fee_coin_decimals)?;
		let entry = coin_entries.get_mut(&fee_coin).ok_or_eyre(format!(
			"Internal error: fee coin {:?} not found in coin_entries during bulk fee computation",
			fee_coin
		))?;
		entry.allowance.is_suff = required_allowance_val.is_zero();
		entry.allowance.required_allowance = required_allowance;
	}

	let total_est_fee_in_fee_coin_decimals =
		denormalize_est_fee_u256(total_est_fee_u256, max_decimals, fee_coin_decimals)?;

	let tot_est_fee = fmt_output(total_est_fee_in_fee_coin_decimals, fee_coin_decimals)?;

	Ok(PreOcpValuesNcwBulk { coin_entries, tot_est_fee })
}

/// Get required allowance value (considering practical case).
///
/// NOTE: If user opts for approving min. amount (instead of `U256::MAX`), then instead of exact
/// `total_spend`, user is asked to approve `total_spend + $10`, so that due to change in network
/// fee, thereby est. fee, user is not prompted to approve again. Here, it's presumed that est. fee
/// won't change by $10, but may be upto $1.
///
/// ## Usage
/// - After the payment page loads, bal & est. fees successfully loaded, then in case of NC, update
///   required allowance on changing amount input. Hence, instead of exact `total_spend`, we ask for
///   `total_spend + $10` during approval.
///
/// ## Arguments
/// - `amount`: entered amount
/// - `coin`
/// - `allowance`: last fetched coin allowance in U256 string.
/// - `est_fee`: last fetched est_fee
/// - `is_fee_incl`
///
/// ## Returns
/// in decimals E.g. "10.124" USDT
pub fn update_req_allowance(
	amount: &str,
	coin: StableCoin,
	allowance: &str,
	est_fee: &str,
	is_fee_incl: bool,
) -> eyre::Result<String> {
	let coin_decimals = coin.decimals();
	let amount_u256 = parse_human_fmt_to_u256(amount, coin_decimals, false)?;
	let est_fee_u256 = parse_human_fmt_to_u256(est_fee, coin_decimals, false)?;

	let mut total_spend = if is_fee_incl { amount_u256 } else { amount_u256 + est_fee_u256 };
	// NOTE: add $10 as safety val, so as to avoid repetitive approval prompt.
	let safety_buffer = U256::from(10_u128) * U256::from(10_u128).pow(U256::from(coin_decimals));
	total_spend = total_spend
		.checked_add(safety_buffer)
		.ok_or_eyre("Overflow while adding safety buffer to total spend")?;

	let allowance = U256::from_str(allowance).wrap_err("Failed to parse allowance")?;
	let req_allowance = if total_spend.gt(&allowance) {
		// We intentionally DO NOT return `total_spend - allowance` here.
		//
		// Why?
		// ERC20 `approve(spender, amount)` overwrites the existing allowance.
		// It does NOT increase it.
		//
		// If we returned only `(total_spend - allowance)` and passed that to
		// `approve()`, the new allowance would be set to only the delta value,
		// which would actually REDUCE the effective allowance instead of making
		// it equal to `total_spend`.
		//
		// Example:
		// - Current allowance = 5
		// - total_spend = 20
		// - total_spend - allowance = 15
		//
		// Calling `approve(spender, 15)` would set allowance to 15,
		// not 20 — which is still insufficient.
		//
		// Therefore, we must approve the full `total_spend` (plus safety buffer),
		// not just the difference.
		//
		// NOTE:
		// If using `increaseAllowance()` instead of `approve()`,
		// then returning `(total_spend - allowance)` would be correct.
		// But since Permit2 / ERC20 approve overwrites,
		// we return the full target allowance.
		total_spend
	} else {
		// Cases:
		// - When `total_spend < approval amt`
		// - When `allowance == U256::MAX`
		U256::ZERO
	};

	fmt_output(req_allowance, coin_decimals)
}

/// Use this instead of `format_units` as it has diff. Err type. \
/// Also, format the zero value as per UI
pub fn fmt_output(value: U256, coin_decimals: u8) -> eyre::Result<String> {
	if value.is_zero() { Ok("0.00".to_string()) } else { Ok(format_units(value, coin_decimals)?) }
}

/// Add the gas-funding and approval cost returned by `request_eth_for_coins_approvals` to a
/// freshly recomputed payment fee.
///
/// The API returns the cost in the default stablecoin's base units (currently USDT, 6 decimals),
/// while the payment fee is denominated in the selected stablecoin. Convert base units before
/// adding so DAI and future coins with different decimal precision recover the same USD amount.
pub fn add_approval_cost_to_est_fee(
	est_fee: &str,
	fee_coin: StableCoin,
	approval_cost: U256,
) -> eyre::Result<String> {
	let fee_decimals = fee_coin.decimals();
	let cost_decimals = StableCoin::default_decimals();
	let mut est_fee_u256 = parse_human_fmt_to_u256(est_fee, fee_decimals, false)?;

	let converted_cost = if fee_decimals == cost_decimals {
		approval_cost
	} else if fee_decimals > cost_decimals {
		let scale = U256::from(10_u8).pow(U256::from((fee_decimals - cost_decimals) as u32));
		approval_cost
			.checked_mul(scale)
			.ok_or_eyre("Overflow while converting approval cost")?
	} else {
		let scale = U256::from(10_u8).pow(U256::from((cost_decimals - fee_decimals) as u32));
		// Round upward so the displayed/recovered fee is never lower than the funded cost.
		approval_cost
			.checked_add(scale - U256::from(1_u8))
			.ok_or_eyre("Overflow while rounding approval cost")? /
			scale
	};

	est_fee_u256 = est_fee_u256
		.checked_add(converted_cost)
		.ok_or_eyre("Overflow while adding approval cost to estimated fee")?;
	fmt_output(est_fee_u256, fee_decimals)
}

/// Get total spend
///
/// ```
/// total_spend = amount + est_fee
/// ```
///
/// ## Usage
/// - In Web app, this is shown to
pub fn total_spend(
	amount: &str,
	est_fee: &str,
	coin: StableCoin,
	is_fee_incl: bool,
) -> eyre::Result<String> {
	if is_fee_incl {
		return Ok(amount.to_owned())
	}

	let coin_decimals = coin.decimals();
	let amt_u256 = parse_human_fmt_to_u256(amount, coin_decimals, false)?;
	let estfee_u256 = parse_human_fmt_to_u256(est_fee, coin_decimals, false)?;

	let total_spend = amt_u256 + estfee_u256;

	fmt_output(total_spend, coin_decimals)
}

/// Validates the amount string and converts it to `U256`.
/// Returns an error if the amount is invalid or zero.
///
/// ## Usage
/// - In base layer, OCP for sanitizing input
/// - In SDK layer, OCP for sanitizing input using `sanitize_and_parse_amount.is_ok()` if value not
///   required. Ideally we need the value in U256 to compare with fetched balance & est fees.
pub fn sanitize_and_parse_amount(
	amount: &str,
	coin: StableCoin,
	use_in_ui: bool,
) -> eyre::Result<U256> {
	let amount_u256 = if use_in_ui {
		parse_payment_amount(amount, coin)?
	} else {
		parse_human_fmt_to_u256(amount, coin.decimals(), false)?
	};
	ensure!(!amount_u256.is_zero(), UfiError::ZeroAmount);
	Ok(amount_u256)
}

/// Validates the amount (with est. fee) against the user's balance.
///
/// ## Arguments
/// - amount: E.g. "1.02345".
/// - ..
///
/// ## Notes
/// - balance & est_fee, is_allowance_zero are fetched via fn `fetch_pre_ocp_balance_and_est_fee`.
/// - In the FE as these are already fetched once & values shown in UI, we don't need to fetch again
///   inside the fn. That's why we are using the fetched values.
///
/// ## Returns
/// - if Ok(true), then valid
/// - if Ok(false), then "Insuficient balance"
/// - if Err(err), then "Max 6 decimals places allowed \nPlease enter a valid amount."
pub fn validate_and_parse_amount(
	amount: &str,
	coin: StableCoin,
	balance: &str,
	est_fee: &str,
	is_fee_incl: bool,
	use_in_ui: bool,
) -> eyre::Result<()> {
	let amount_u256 = sanitize_and_parse_amount(amount, coin, use_in_ui)?;
	let balance_u256: U256 = parse_units(balance, coin.decimals())?.into();
	let est_fee_u256: U256 = parse_units(est_fee, coin.decimals())?.into();

	let total_amount_u256 = if is_fee_incl {
		amount_u256
	} else {
		amount_u256.checked_add(est_fee_u256).ok_or_eyre(
			"Calculation error: Failed to add amount and estimated fees — possible overflow",
		)?
	};

	ensure!(total_amount_u256.le(&balance_u256), UfiError::InsufficientBalance);

	Ok(())
}

/// Validate and parse amount without sanitization.
///
/// ## Arguments
/// - amount: U256 string i.e. use "102345" for "1.02345" USDT.
/// - ..
///
/// ## Usage
/// - FliQPay page where we separately sanitize the amount synchronously
pub fn validate_and_parse_amount_wo_sanitize(
	amount: &str,
	coin: StableCoin,
	balance: &str,
	est_fee: &str,
	is_fee_incl: bool,
) -> eyre::Result<()> {
	let amount_u256 = amount.parse::<U256>()?;
	let balance_u256: U256 = parse_units(balance, coin.decimals())?.into();
	let est_fee_u256: U256 = parse_units(est_fee, coin.decimals())?.into();

	let total_amount_u256 = if is_fee_incl {
		amount_u256
	} else {
		amount_u256.checked_add(est_fee_u256).ok_or_eyre(
			"Calculation error: Failed to add amount and estimated fees — possible overflow",
		)?
	};

	ensure!(total_amount_u256.le(&balance_u256), UfiError::InsufficientBalance);

	Ok(())
}

/// Calculate the no. of worker threads per CPU
pub fn calculate_worker_threads() -> usize {
	let cpu_cores = num_cpus::get(); // Get available CPU cores
	if cpu_cores == 1 {
		4 // Minimum 4 threads for async tasks
	} else {
		cpu_cores * 8 // 8x multiplier for high concurrency
	}
}

/// ## Usage
/// - to set start_timestamp_us in API plan.
#[cfg(target_arch = "wasm32")]
pub fn now_timestamp_us() -> i64 {
	(js_sys::Date::now() * 1000.0) as i64
}

/// ## Usage
/// - for build_headers test inside api-plug lib.rs file.
#[cfg(not(target_arch = "wasm32"))]
pub fn now_timestamp_us() -> i64 {
	chrono::Utc::now().timestamp_micros()
}

/// ## Usage
/// - It's run inside dioxus app whenever sdk.__(..) is used. We internally use `with_auth` in each
///   fn, defined in api-plug fns.
#[cfg(target_arch = "wasm32")]
pub fn now_timestamp_secs() -> u64 {
	(js_sys::Date::now() / 1000.0) as u64
}

/// ## Usage
/// - for build_headers test inside api-plug lib.rs file.
#[cfg(not(target_arch = "wasm32"))]
pub fn now_timestamp_secs() -> u64 {
	chrono::Utc::now().timestamp() as u64
}

/// Generate a unique id (UID) using blake3 (fast).
///
/// ## Parameters
/// - user_id: User ID from Telegram bot API or google/meta/apple login
/// - timestamp_as_micros: Timestamp as microseconds
///
/// ## Usage
/// - Generate UID.
/// - Generate PID. The purpose is to not share the UID or user_id, but instead share PID.
///   - This ID is used to send payment notification 🔔 to Payer after it shares its PID with the
///     recipient.
/// - Generate Onchain/Offchain Pay receipt ID.
///
///
/// ## Returns
/// Output: uid (last 12 bytes hex string)
///
/// NOTE: 12 bytes because of shorter size like document id to fetch faster. Can't rely on
/// MongoDB's document id as it's specific to MongoDB. In future, if we change to some other
/// scalable DB like Cassandra, Scylla, then we have to come up with our own uid generation
/// algorithm.
pub fn generate_unique_id(user_id: &str, timestamp_as_micros: i64) -> String {
	let input = format!("{}{}", user_id, timestamp_as_micros);
	let hash = blake3::hash(input.as_bytes()).to_hex().to_string();
	// Extract the last 12 bytes (96 bits) of the hash
	hash[40..].to_string()
}

/// Test
/// ```sh
/// RUSTFLAGS="-Awarnings" cargo t -p unifi-sdk-primitives -F utils -- utils::tests --show-output
/// ```
#[cfg(test)]
mod tests {
	use super::*;

	fn ncw_batch_quote(
		params: &PreOcpValuesNcwParamsBulk,
		items: &[(StableCoin, String)],
	) -> eyre::Result<Vec<PaymentCoinQuote>> {
		payment_batch_quote(
			PaymentBatchParams::NonCustodial { chain: ChainName::Sepolia, params },
			items,
		)
	}

	fn custodial_prefetch_fixture() -> PreOcpValuesBulk {
		PreOcpValuesBulk {
			gas_price: 1_000_000_000,
			gas_token_price: 3000.0,
			coin_entries: StableCoin::all()
				.iter()
				.map(|coin| crate::types::PreOcpValuesBulkCoin {
					coin: *coin,
					balance: "100".into(),
					allowance: U256::MAX.to_string(),
					coin_price: 1.0,
				})
				.collect(),
			..Default::default()
		}
	}

	fn custodial_batch_quote(
		params: &PreOcpValuesBulk,
		items: &[(StableCoin, String)],
	) -> eyre::Result<Vec<PaymentCoinQuote>> {
		payment_batch_quote(
			PaymentBatchParams::Custodial { chain: ChainName::Sepolia, params },
			items,
		)
	}

	fn ncw_prefetch_fixture() -> PreOcpValuesNcwParamsBulk {
		PreOcpValuesNcwParamsBulk {
			gas_price: 1_000_000_000,
			gas_token_price: 3000.0,
			coin_entries: StableCoin::all()
				.iter()
				.map(|coin| crate::types::PreOcpValuesNcwParamsBulkCoin {
					coin: *coin,
					balance: "100".into(),
					allowance: U256::MAX.to_string(),
					coin_price: 1.0,
				})
				.collect(),
			..Default::default()
		}
	}

	#[test]
	fn payment_batch_empty_drafts_estimate_fees_without_allowance_underflow() {
		let rows = ncw_batch_quote(
			&ncw_prefetch_fixture(),
			&[(StableCoin::USDT, String::new()), (StableCoin::USDC, String::new())],
		)
		.unwrap();
		assert_eq!(rows.len(), 2);
		assert!(rows.iter().all(|row| row.shortfall.is_none()));
	}

	#[test]
	fn payment_batch_uses_fee_coin_price_for_every_transfer() {
		let mut portfolio = ncw_prefetch_fixture();
		let items = [(StableCoin::USDT, "1".into()), (StableCoin::DAI, "1".into())];
		let before = ncw_batch_quote(&portfolio, &items).unwrap();
		portfolio
			.coin_entries
			.iter_mut()
			.find(|entry| entry.coin == StableCoin::USDT)
			.unwrap()
			.coin_price = 0.5;
		let after = ncw_batch_quote(&portfolio, &items).unwrap();
		assert_eq!(before, after, "A non-fee coin's price cannot change fees charged in DAI");
	}

	#[test]
	fn payment_batch_fee_crossing_allowance_includes_approval_cost() {
		let mut portfolio = ncw_prefetch_fixture();
		let items = [(StableCoin::USDT, "10".into())];
		let without_approval = ncw_batch_quote(&portfolio, &items).unwrap().remove(0);
		portfolio
			.coin_entries
			.iter_mut()
			.find(|entry| entry.coin == StableCoin::USDT)
			.unwrap()
			.allowance = "10000000".into();
		let with_approval = ncw_batch_quote(&portfolio, &items).unwrap().remove(0);
		assert!(
			parse_payment_amount(&with_approval.est_fee, StableCoin::USDT).unwrap() >
				parse_payment_amount(&without_approval.est_fee, StableCoin::USDT).unwrap()
		);
	}

	#[test]
	fn payment_batch_same_coin_aggregates_before_balance_check() {
		let rows = ncw_batch_quote(
			&ncw_prefetch_fixture(),
			&[(StableCoin::USDT, "60".into()), (StableCoin::USDT, "50".into())],
		)
		.unwrap();
		assert_eq!(rows.len(), 1);
		assert_eq!(
			parse_payment_amount(&rows[0].amount, StableCoin::USDT).unwrap(),
			U256::from(110_000_000)
		);
		assert!(rows[0].shortfall.is_some());
	}

	#[test]
	fn payment_batch_mixed_coins_only_charge_last_coin_for_ncw_fee() {
		let rows = ncw_batch_quote(
			&ncw_prefetch_fixture(),
			&[(StableCoin::USDT, "100".into()), (StableCoin::DAI, "100".into())],
		)
		.unwrap();
		let usdt = rows.iter().find(|row| row.coin == StableCoin::USDT).unwrap();
		let dai = rows.iter().find(|row| row.coin == StableCoin::DAI).unwrap();
		assert!(usdt.shortfall.is_none());
		assert!(parse_payment_amount(&usdt.est_fee, usdt.coin).unwrap().is_zero());
		assert!(dai.shortfall.is_some());
		assert!(parse_payment_amount(&dai.est_fee, dai.coin).unwrap() > U256::ZERO);
	}

	#[test]
	fn payment_batch_marks_unfunded_amounts_from_top_to_bottom_per_coin() {
		let mut params = custodial_prefetch_fixture();
		for entry in &mut params.coin_entries {
			entry.balance = if entry.coin == StableCoin::USDT { "260.155420" } else { "0" }.into();
		}
		let rows = custodial_batch_quote(
			&params,
			&[
				(StableCoin::USDT, "200".into()),
				(StableCoin::USDC, "100".into()),
				(StableCoin::USDC, "1.888888".into()),
				(StableCoin::DAI, "0".into()),
				(StableCoin::USDC, "0".into()),
				(StableCoin::USDC, String::new()),
			],
		)
		.unwrap();
		let usdc = rows.iter().find(|row| row.coin == StableCoin::USDC).unwrap();
		assert_eq!(usdc.insufficient_recipients, vec![1, 2]);
		assert!(
			rows.iter()
				.filter(|row| row.coin != StableCoin::USDC)
				.all(|row| row.insufficient_recipients.is_empty())
		);
	}

	#[test]
	fn payment_batch_reserves_fee_before_assigning_available_balance() {
		let mut params = custodial_prefetch_fixture();
		let items = [(StableCoin::USDT, "50".into()), (StableCoin::USDT, "50".into())];
		params.coin_entries[0].balance = "100.552000".into();
		let exact = custodial_batch_quote(&params, &items).unwrap();
		assert!(exact[0].insufficient_recipients.is_empty());
		params.coin_entries[0].balance = "100.551999".into();
		let short = custodial_batch_quote(&params, &items).unwrap();
		assert_eq!(short[0].insufficient_recipients, vec![1]);
		assert!(short[0].shortfall.is_some());
	}

	#[test]
	fn payment_batch_only_marks_later_unfunded_ncw_amounts_and_clears_after_edit() {
		let params = ncw_prefetch_fixture();
		let mut items = [
			(StableCoin::USDT, "60".into()),
			(StableCoin::DAI, "1".into()),
			(StableCoin::USDT, "50".into()),
			(StableCoin::USDT, "1".into()),
		];
		let short = ncw_batch_quote(&params, &items).unwrap();
		let usdt = short.iter().find(|row| row.coin == StableCoin::USDT).unwrap();
		assert_eq!(usdt.insufficient_recipients, vec![2, 3]);
		items[2].1 = "10".into();
		let fixed = ncw_batch_quote(&params, &items).unwrap();
		assert!(fixed.iter().all(|row| row.insufficient_recipients.is_empty()));
		assert!(fixed.iter().all(|row| row.shortfall.is_none()));
	}

	#[test]
	fn payment_batch_custodial_recalculates_one_fee_with_recipient_count() {
		let mut portfolio = custodial_prefetch_fixture();
		portfolio.coin_entries[0].balance = "100.552000".into();
		let coin = portfolio.coin_entries[0].coin;
		let items = [(coin, "50".into()), (coin, "50".into())];
		let row = custodial_batch_quote(&portfolio, &items).unwrap().remove(0);
		assert!(row.shortfall.is_none());
		assert_eq!(
			parse_payment_amount(&row.est_fee, coin).unwrap(),
			parse_payment_amount("0.552", coin).unwrap()
		);
		let single = custodial_batch_quote(&portfolio, &items[..1]).unwrap();
		assert_eq!(single[0].est_fee, "0.276000");
		portfolio.coin_entries[0].balance = "100.551999".into();
		assert!(custodial_batch_quote(&portfolio, &items).unwrap()[0].shortfall.is_some());
	}

	#[test]
	fn payment_batch_custodial_mixed_coins_check_balances_and_fees_independently() {
		let mut params = custodial_prefetch_fixture();
		params
			.coin_entries
			.iter_mut()
			.find(|entry| entry.coin == StableCoin::USDC)
			.unwrap()
			.balance = "1".into();
		let items = [(StableCoin::USDT, "20".into()), (StableCoin::USDC, "1".into())];
		let rows = custodial_batch_quote(&params, &items).unwrap();
		assert!(
			rows.iter()
				.find(|row| row.coin == StableCoin::USDT)
				.unwrap()
				.shortfall
				.is_none()
		);
		assert!(
			rows.iter()
				.find(|row| row.coin == StableCoin::USDC)
				.unwrap()
				.shortfall
				.is_some()
		);
		params.coin_entries.retain(|entry| entry.coin != StableCoin::USDC);
		assert!(custodial_batch_quote(&params, &items).is_err());
	}

	#[test]
	fn custodial_batch_approval_cost_is_added_once_per_coin() {
		let mut params = custodial_prefetch_fixture();
		params.coin_entries[0].allowance = "0".into();
		let coin = params.coin_entries[0].coin;
		let single = custodial_batch_quote(&params, &[(coin, "1".into())]).unwrap();
		let double =
			custodial_batch_quote(&params, &[(coin, "1".into()), (coin, "1".into())]).unwrap();
		assert_eq!(single[0].est_fee, "0.693450");
		assert_eq!(double[0].est_fee, "0.969450");
	}

	#[test]
	fn custodial_batch_charges_only_fee_coin_and_uses_its_price() {
		let mut params = custodial_prefetch_fixture();
		let items = [(StableCoin::USDT, "10".into()), (StableCoin::DAI, "10".into())];
		let before = custodial_batch_quote(&params, &items).unwrap();
		params
			.coin_entries
			.iter_mut()
			.find(|entry| entry.coin == StableCoin::USDT)
			.unwrap()
			.coin_price = 0.5;
		let after = custodial_batch_quote(&params, &items).unwrap();
		assert_eq!(before, after);
		let usdt = after.iter().find(|row| row.coin == StableCoin::USDT).unwrap();
		let dai = after.iter().find(|row| row.coin == StableCoin::DAI).unwrap();
		assert!(parse_payment_amount(&usdt.est_fee, usdt.coin).unwrap().is_zero());
		assert_eq!(
			parse_payment_amount(&dai.est_fee, dai.coin).unwrap(),
			parse_payment_amount("0.552", dai.coin).unwrap()
		);
		assert!(after.iter().all(|row| row.shortfall.is_none()));
	}

	#[test]
	fn custodial_batch_recalculates_selected_approvals_and_fee_coin_price() {
		let mut params = custodial_prefetch_fixture();
		for entry in &mut params.coin_entries {
			entry.allowance = "0".into();
		}
		let items = [(StableCoin::USDT, "1".into()), (StableCoin::USDC, "1".into())];
		let both = custodial_batch_quote(&params, &items).unwrap();
		let fee = both.iter().find(|row| row.coin == StableCoin::USDC).unwrap();
		assert_eq!(fee.est_fee, "1.386900");
		// Removing a coin also removes its approval cost. Unused DAI is never approved.
		let remaining = custodial_batch_quote(&params, &items[1..]).unwrap();
		assert_eq!(remaining[0].est_fee, "0.693450");
		params
			.coin_entries
			.iter_mut()
			.find(|entry| entry.coin == StableCoin::USDC)
			.unwrap()
			.coin_price = 0.5;
		let repriced = custodial_batch_quote(&params, &items[1..]).unwrap();
		assert_eq!(repriced[0].est_fee, "1.386900");
	}

	#[test]
	fn custodial_batch_rounds_up_once_after_summing_gas() {
		let mut params = custodial_prefetch_fixture();
		params.gas_price = 1;
		let rows = custodial_batch_quote(
			&params,
			&[(StableCoin::USDT, "1".into()), (StableCoin::USDT, "1".into())],
		)
		.unwrap();
		assert_eq!(rows[0].est_fee, "0.000001");
	}

	#[test]
	fn custodial_batch_rejects_invalid_metadata_and_gas_overflow() {
		let items = [(StableCoin::USDT, "1".into())];
		let mut params = custodial_prefetch_fixture();
		params.gas_token_price = f64::NAN;
		assert!(custodial_batch_quote(&params, &items).is_err());
		params.gas_token_price = 3000.0;
		params.coin_entries[0].coin_price = 0.0;
		assert!(custodial_batch_quote(&params, &items).is_err());
		params.coin_entries[0].coin_price = 1.0;
		params.coin_entries[0].allowance = "invalid".into();
		assert!(custodial_batch_quote(&params, &items).is_err());
		params.coin_entries[0].allowance = U256::MAX.to_string();
		params.gas_price = u128::MAX;
		assert!(custodial_batch_quote(&params, &items).is_err());
	}

	#[test]
	fn payment_batch_ignores_unused_coins_and_rejects_missing_data() {
		let mut portfolio = ncw_prefetch_fixture();
		let items = [(StableCoin::USDC, "1".into())];
		assert_eq!(ncw_batch_quote(&portfolio, &items).unwrap().len(), 1);
		portfolio.coin_entries.retain(|entry| entry.coin != StableCoin::USDC);
		assert!(ncw_batch_quote(&portfolio, &items).is_err());
	}

	#[test]
	fn payment_batch_rejects_malformed_precision_and_overflow() {
		for amount in [
			"1.2.3",
			"1.abc",
			"-1",
			"NaN",
			"1e5",
			"1.0000001",
			"9999999999999999999999999999999999999999999999999999999999999999999999999999999",
		] {
			assert!(
				ncw_batch_quote(&ncw_prefetch_fixture(), &[(StableCoin::USDT, amount.into())])
					.is_err(),
				"{amount}"
			);
		}
		assert_eq!(
			parse_payment_amount("1.000000000000000001", StableCoin::DAI).unwrap(),
			U256::from(1_000_000_000_000_000_001_u128)
		);
	}

	#[test]
	fn ui_amount_validation_preserves_each_coins_decimal_error() {
		for coin in StableCoin::all() {
			let valid = format!("1.{}", "5".repeat(coin.decimals() as usize));
			assert!(sanitize_and_parse_amount(&valid, *coin, true).is_ok());
			let invalid = format!("{valid}5");
			let error =
				validate_and_parse_amount(&invalid, *coin, "100", "0", false, true).unwrap_err();
			assert!(
				matches!(error.downcast_ref::<UfiError>(), Some(UfiError::MaxDecimalsReached(decimals)) if *decimals == coin.decimals())
			);
		}
	}

	#[test]
	fn ui_amount_validation_is_checked_and_preserves_zero_and_balance_errors() {
		let coin = StableCoin::USDT;
		assert_eq!(sanitize_and_parse_amount(".1", coin, true).unwrap(), U256::from(100_000));
		for invalid in ["1.abc", "1.2.3", "NaN", "-1", ".", "1e5"] {
			assert!(sanitize_and_parse_amount(invalid, coin, true).is_err(), "{invalid}");
		}
		assert!(sanitize_and_parse_amount(&U256::MAX.to_string(), coin, true).is_err());
		let zero = validate_and_parse_amount("0", coin, "100", "0", false, true).unwrap_err();
		assert!(matches!(zero.downcast_ref::<UfiError>(), Some(UfiError::ZeroAmount)));
		let short = validate_and_parse_amount("100", coin, "100", "1", false, true).unwrap_err();
		assert!(matches!(short.downcast_ref::<UfiError>(), Some(UfiError::InsufficientBalance)));
	}

	#[test]
	fn payment_batch_fee_changes_with_count_and_selected_fee_coin() {
		let portfolio = ncw_prefetch_fixture();
		let one = ncw_batch_quote(&portfolio, &[(StableCoin::USDT, "1".into())]).unwrap();
		let two = ncw_batch_quote(
			&portfolio,
			&[(StableCoin::USDT, "1".into()), (StableCoin::USDT, "1".into())],
		)
		.unwrap();
		assert!(
			parse_payment_amount(&two[0].est_fee, StableCoin::USDT).unwrap() >
				parse_payment_amount(&one[0].est_fee, StableCoin::USDT).unwrap()
		);
		let switched = ncw_batch_quote(
			&portfolio,
			&[(StableCoin::DAI, "1".into()), (StableCoin::USDT, "1".into())],
		)
		.unwrap();
		assert!(
			parse_payment_amount(
				&switched.iter().find(|q| q.coin == StableCoin::DAI).unwrap().est_fee,
				StableCoin::DAI
			)
			.unwrap()
			.is_zero()
		);
	}

	#[test]
	fn test_req_allowance() {
		// Test with USDT token
		assert_eq!(
			update_req_allowance("1", StableCoin::USDT, "0", "0.112192", false).unwrap(),
			"11.112192"
		);
		assert_eq!(
			update_req_allowance("1", StableCoin::USDT, "0", "0.112192", true).unwrap(),
			"11.000000"
		);

		// Test with DAI token
		assert_eq!(
			update_req_allowance("1", StableCoin::DAI, "0", "0.112192", false).unwrap(),
			"11.112192000000000000"
		);
		assert_eq!(
			update_req_allowance("1", StableCoin::DAI, "0", "0.112192", true).unwrap(),
			"11.000000000000000000"
		);
	}

	#[test]
	fn approval_cost_uses_the_fee_coins_decimals() {
		let approval_cost = U256::from(125_000_u64); // $0.125 in default 6-decimal units.

		assert_eq!(
			add_approval_cost_to_est_fee("0.25", StableCoin::USDT, approval_cost).unwrap(),
			"0.375000"
		);
		assert_eq!(
			add_approval_cost_to_est_fee("0.25", StableCoin::DAI, approval_cost).unwrap(),
			"0.375000000000000000"
		);
	}

	mod denormalize_est_fee_u256_tests {
		use super::*;

		#[test]
		// fn test_denormalize_est_fee_u256_same_decimals_keeps_value_unchanged() {
		fn same_decimals_keeps_value_unchanged() {
			let value = parse_human_fmt_to_u256("0.025762145346208266", 18, false).unwrap();

			assert_eq!(denormalize_est_fee_u256(value, 18, 18).unwrap(), value);
			assert_eq!(fmt_output(value, 18).unwrap(), "0.025762145346208266");
		}

		#[test]
		fn from_18_to_6_truncates_for_display() {
			let value_18 = parse_human_fmt_to_u256("0.050494861262689543", 18, false).unwrap();
			let value_6 = denormalize_est_fee_u256(value_18, 18, 6).unwrap();

			assert_eq!(fmt_output(value_18, 18).unwrap(), "0.050494861262689543");
			assert_eq!(value_6, U256::from(50_494_u128));
			assert_eq!(fmt_output(value_6, 6).unwrap(), "0.050494");
		}

		#[test]
		fn bulkpay_examples_match_fee_coin_decimals() {
			let cases = [
				("0.025762145346208266", 18_u8, "0.025762145346208266"),
				("0.050494861262689543", 6_u8, "0.050494"),
				("0.075226291894034316", 6_u8, "0.075226"),
				("0.099961153156723860", 6_u8, "0.099961"),
				("0.130874728533468004", 6_u8, "0.130874"),
			];

			for (raw_est_fee, fee_coin_decimals, expected_display) in cases {
				let value_18 = parse_human_fmt_to_u256(raw_est_fee, 18, false).unwrap();
				let denormalized =
					denormalize_est_fee_u256(value_18, 18, fee_coin_decimals).unwrap();

				assert_eq!(fmt_output(denormalized, fee_coin_decimals).unwrap(), expected_display);
			}
		}
	}
}
