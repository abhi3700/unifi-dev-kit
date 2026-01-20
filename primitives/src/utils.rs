use crate::{
	errors::UfiError,
	types::{Coin, GasEstimate, PreOcpPayload, PreOcpValuesNcwParams, StableCoin},
};
use alloy_primitives::{
	U256,
	utils::{format_units, parse_units},
};
use eyre::{Context, OptionExt, ensure};
use std::str::FromStr;

/// Format any num (in U256 String) to Decimal formatted considering coin's decimals.
pub fn fmt_value(num_in_u256_str: &str, coin: StableCoin) -> eyre::Result<String> {
	let value = format_units(U256::from_str(num_in_u256_str)?, coin.decimals())?;
	Ok(value)
}

pub fn is_value_gte(num_in_u256_str: &str, amount: &str, coin_decimals: u8) -> eyre::Result<bool> {
	Ok(num_in_u256_str.parse::<U256>()? >= parse_human_fmt_to_u256(amount, coin_decimals, true)?)
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
///   - `false`: for precision during math calculation, value might be much bigger than coin
///     decimals.
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

/// Compute Est. fee for NC Pay.
///
///
/// ## Notes
/// - This fn can't be `async` bcoz on change of amount on UI, the text box err (if any) should show
///   synchronously.
/// - If the fee is:
///   - excl: We just have the entered amount. So, there is a `est fees` computed after considering
///     only amount & then `amount + est_fee` is checked against allowance, etc.
///   - incl: We have to do the maths once & then
///
/// ## Arguments
/// - `payload`: selected {coin, chain}
/// - `tot_amount`: sum of [amount1, amount2, .., admin_fee]. If the value is "112.432432" USDT,
///   then parse as u256 str i.e. "112432432".
/// - `pre_ocp_values`: params (from fn: `prefetch_ncw_balance_fee_params`) for calculating est fees
///   synchronously.
/// - `is_fee_incl`
///
/// ## Returns
/// - `is_coin_allowance_suff`: NOTE: due to this field returned, we don't have to convert the
///   formatted required allowance to U256 for zero check.
///   - true: coin allowance is sufficient. Hence, all ok from amount box side.
///   - false: coin allowance is insufficient. So, an err is shown saying "Please approve X amount"
/// - required_allowance: X value is to be approved by payer to Permit2. E.g. `21.34545` USDT or
///   "0.00" USDT.
/// - formatted est fees. E.g. `0.132433` USDT or "0.00" USDT.
pub fn compute_est_fee_ncw(
	payload: PreOcpPayload,
	tot_amount_u256: &str,
	pre_ocp_values: &PreOcpValuesNcwParams,
	is_fee_incl: bool,
) -> eyre::Result<(bool, String, String)> {
	// 1. Destructure and Parse Inputs immediately
	let PreOcpPayload { coin, chain } = payload;
	let PreOcpValuesNcwParams {
		allowance: allowance_str,
		balance: balance_str,
		gas_price,
		gas_token_price,
		coin_price,
	} = pre_ocp_values;

	let coin_decimals = coin.decimals();
	let allowance = U256::from_str(allowance_str).wrap_err("Failed to parse allowance")?;
	let mut tot_amount = U256::from_str(tot_amount_u256).wrap_err("Failed to parse amount")?;
	let balance = parse_human_fmt_to_u256(balance_str, coin_decimals, true)?;

	// 2. Early Balance Check (Fail fast)
	if tot_amount.gt(&balance) {
		return Err(UfiError::InsufficientBalance.into())
	}

	// 3. Pre-calculate Constants
	let GasEstimate { approve, permit_transfer_from, .. } = chain.get_gas_usage_limit(coin);

	// Optimization: Calculate price denominator once.
	// Formula: coin_price * 10^(gas_coin_decimals)
	let gas_coin_decimals = Coin::chain_to_gas_coin(chain).decimals() as i32;
	let price_denom = coin_price * 10f64.powi(gas_coin_decimals);

	// 4. Define Calculation Logic (Closure to handle repetition)
	// Returns: (Calculated Fee U256, Is Allowance Sufficient)
	let calc_snapshot = |target_amt: U256| -> eyre::Result<(U256, U256)> {
		// NOTE: For C Pay, allowance is either 0 or MAX unlike in NC Pay.
		let is_suff = !allowance.is_zero() && allowance.ge(&target_amt);

		let (est_gas_usage, required_allowance_val) = if is_suff {
			(permit_transfer_from, U256::ZERO)
		} else {
			(approve + permit_transfer_from, target_amt - allowance)
		};

		// fee = (gas_usage * gas_price * gas_token_price) / (coin_price * 10^decimals)
		let est_gas_fee = (est_gas_usage * gas_price) as f64;
		let est_fee_f64 = est_gas_fee * gas_token_price / price_denom;
		let est_fee_u256 = parse_human_fmt_to_u256(&est_fee_f64.to_string(), coin_decimals, false)?;

		Ok((est_fee_u256, required_allowance_val))
	};

	// 5. Initial Calculation
	let (mut est_fee_u256, mut required_allowance_val) = calc_snapshot(tot_amount)?;

	// 6. Conditional Re-calculation (If fee is excluded, total amount increases)
	if !is_fee_incl {
		tot_amount += est_fee_u256;
		if tot_amount.gt(&balance) {
			return Err(UfiError::InsufficientBalance.into())
		}

		// Check if adding the fee pushed us over the allowance threshold
		(est_fee_u256, required_allowance_val) = calc_snapshot(tot_amount)?;
	}

	// 7. Format Output
	let is_suff = required_allowance_val.is_zero();
	let required_allowance_val_fmt = fmt_output(required_allowance_val, coin_decimals)?;
	let est_fee_fmt = fmt_output(est_fee_u256, coin_decimals)?;

	Ok((is_suff, required_allowance_val_fmt, est_fee_fmt))
}

pub fn fmt_output(value: U256, coin_decimals: u8) -> eyre::Result<String> {
	if value.is_zero() { Ok("0.00".to_string()) } else { Ok(format_units(value, coin_decimals)?) }
}

/// Validates the amount string and converts it to `U256`.
/// Returns an error if the amount is invalid or zero.
///
/// ## Usage
/// - In base layer, OCP for sanitizing input
/// - In SDK layer, OCP for sanitizing input using `sanitize_and_parse_amount.is_ok()` if value not
///   required. Ideally we need the value in U256 to compare with fetched balance & est fees.
pub fn sanitize_and_parse_amount(amount: &str, coin: StableCoin) -> eyre::Result<U256> {
	let amount_u256 = parse_human_fmt_to_u256(amount, coin.decimals(), true)?;
	ensure!(!amount_u256.is_zero(), UfiError::ZeroAmount);
	Ok(amount_u256)
}

/// Validates the amount (with est. fee) against the user's balance.
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
) -> eyre::Result<()> {
	let amount_u256 = sanitize_and_parse_amount(amount, coin)?;
	let balance_u256: U256 = parse_units(balance, coin.decimals())?.into();
	let est_fee_u256: U256 = parse_units(est_fee, coin.decimals())?.into();

	let total_amount_u256 = amount_u256.checked_add(est_fee_u256).ok_or_eyre(
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
	est_fee: &str,
) -> eyre::Result<()> {
	let amount_u256 = amount.parse::<U256>()?;
	let balance_u256: U256 = parse_units(balance, coin.decimals())?.into();
	let est_fee_u256: U256 = parse_units(est_fee, coin.decimals())?.into();

	let total_amount_u256 = amount_u256.checked_add(est_fee_u256).ok_or_eyre(
		"Calculation error: Failed to add amount and estimated fees — possible overflow",
	)?;

	ensure!(total_amount_u256.le(&balance_u256), UfiError::InsufficientBalance);

	Ok(())
}
