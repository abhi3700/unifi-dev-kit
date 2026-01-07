use crate::{
	errors::UfiError,
	types::{ChainName, Coin, GasEstimate, PreOcpValuesNcw, StableCoin},
};
use alloy_primitives::{
	U256,
	utils::{format_units, parse_units},
};
use eyre::{OptionExt, ensure};
use std::str::FromStr;

/// Format any num (in U256 String) to Decimal formatted considering coin's decimals.
pub fn fmt_value(num_in_u256_str: &str, coin: StableCoin) -> eyre::Result<String> {
	let value = format_units(U256::from_str(num_in_u256_str)?, coin.decimals())?;
	Ok(value)
}

pub fn is_value_gte(num_in_u256_str: &str, amount: &str, coin_decimals: u8) -> eyre::Result<bool> {
	Ok(num_in_u256_str.parse::<U256>()? >= parse_human_fmt_to_u256(amount, coin_decimals)?)
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
pub fn parse_human_fmt_to_u256(value: &str, coin_decimals: u8) -> eyre::Result<U256> {
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
		ensure!(frac.len() <= coin_decimals as usize, UfiError::MaxDecimalsReached(coin_decimals));
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

/// Convert from f64 to U256 without rounding off.
/// Normally, "3.819636527426028" returns "4" using `U256::from(..)`. To prevent this
/// rounding-off, we need to truncate to only valid decimals like 6 for USDT, 18 for DAI, etc.
/// TODO: Test
pub fn f64_to_u256_dec(input: f64, decimals: u8) -> U256 {
	// Convert decimals to a power of 10
	let multiplier = 10u64.pow(decimals as u32) as f64;

	// Multiply the input by the multiplier to shift the decimal point
	let scaled_input = input * multiplier;

	// Convert the scaled float to an integer type safely
	let integer_part = scaled_input.trunc() as u128;

	// Convert the integer to U256
	U256::from(integer_part)
}

/// tot_amount = amount + est_fees.
///
/// NOTE: This fn can't be `async` bcoz on change of amount on UI, the text box err (if any) should
/// show synchronously.
///
/// ## Returns
/// - formatted est fees. E.g. `0.132433` USDT or "0.00" USDT.
///
/// TODO: Test
pub fn compute_est_fees_ncw(
	coin: StableCoin,
	chain: ChainName,
	tot_amount: &str,
	pre_ocp_values: PreOcpValuesNcw,
) -> eyre::Result<String> {
	let PreOcpValuesNcw { allowance, balance, gas_price, gas_token_price, coin_price } =
		pre_ocp_values;
	let coin_decimals = coin.decimals();
	let amount = parse_human_fmt_to_u256(tot_amount, coin_decimals)?;
	let balance = parse_human_fmt_to_u256(&balance, coin_decimals)?;

	if amount.gt(&balance) {
		return Err(UfiError::InsufficientBalance.into())
	}

	let GasEstimate { approve, permit_transfer_from, .. } = chain.get_gas_usage_limit(coin);

	let allowance = U256::from_str(&allowance)?;
	let est_gas_usage = if allowance.is_zero() || allowance.lt(&amount) {
		approve + permit_transfer_from
	} else {
		permit_transfer_from
	};

	// fee = gas * gas_price
	let est_gas_fee = (est_gas_usage * gas_price) as f64;
	let est_fee_f64 = est_gas_fee * gas_token_price /
		(coin_price * 10_i32.pow(Coin::chain_to_gas_coin(chain).decimals() as u32) as f64);
	let est_fee_u256 = f64_to_u256_dec(est_fee_f64, coin_decimals);
	let est_fee_formatted = if est_fee_u256.ne(&U256::ZERO) {
		format_units(est_fee_u256, coin_decimals)?
	} else {
		"0.00".to_string()
	};

	Ok(est_fee_formatted)
}

/// Validates the amount string and converts it to `U256`.
/// Returns an error if the amount is invalid or zero.
///
/// ## Usage
/// - In base layer, OCP for sanitizing input
/// - In SDK layer, OCP for sanitizing input using `sanitize_and_parse_amount.is_ok()` if value not
///   required. Ideally we need the value in U256 to compare with fetched balance & est fees.
pub fn sanitize_and_parse_amount(amount: &str, coin: StableCoin) -> eyre::Result<U256> {
	let amount_u256 = parse_human_fmt_to_u256(amount, coin.decimals())?;
	ensure!(!amount_u256.is_zero(), UfiError::ZeroAmount);
	Ok(amount_u256)
}

/// Validates the amount (with est. fees) against the user's balance.
///
/// ## Notes
/// - balance & est_fees, is_allowance_zero are fetched via fn `fetch_pre_ocp_balance_and_est_fees`.
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
	est_fees: &str,
) -> eyre::Result<()> {
	let amount_u256 = sanitize_and_parse_amount(amount, coin)?;
	let balance_u256: U256 = parse_units(balance, coin.decimals())?.into();
	let est_fees_u256: U256 = parse_units(est_fees, coin.decimals())?.into();

	let total_amount_u256 = amount_u256.checked_add(est_fees_u256).ok_or_eyre(
		"Calculation error: Failed to add amount and estimated fees — possible overflow",
	)?;

	ensure!(total_amount_u256.le(&balance_u256), UfiError::InsufficientBalance);

	Ok(())
}

/// Validate and parse amount without sanitization.
///
/// ## Usage
/// - FliQPay page.
pub fn validate_and_parse_amount_wo_sanitize(
	amount: &str,
	coin: StableCoin,
	balance: &str,
	est_fees: &str,
) -> eyre::Result<()> {
	let amount_u256 = amount.parse::<U256>()?;
	let balance_u256: U256 = parse_units(balance, coin.decimals())?.into();
	let est_fees_u256: U256 = parse_units(est_fees, coin.decimals())?.into();

	let total_amount_u256 = amount_u256.checked_add(est_fees_u256).ok_or_eyre(
		"Calculation error: Failed to add amount and estimated fees — possible overflow",
	)?;

	ensure!(total_amount_u256.le(&balance_u256), UfiError::InsufficientBalance);

	Ok(())
}
