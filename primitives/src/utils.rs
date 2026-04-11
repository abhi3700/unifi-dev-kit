use crate::{
	errors::UfiError,
	types::{
		ChainName, GasEstimate, PreOcpPayload, PreOcpValuesNcwBulk, PreOcpValuesNcwBulkCoin,
		PreOcpValuesNcwParams, PreOcpValuesNcwParamsBulk, StableCoin,
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
/// - formatted est fees. E.g. `0.132433` USDT or "0.00" USDT.
pub fn compute_est_fee_ncw(
	payload: PreOcpPayload,
	amt_or_tot_amount: &str,
	pre_ocp_values: &PreOcpValuesNcwParams,
	is_fee_incl: bool,
	use_in_ui: bool,
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
	let GasEstimate { approve, permit_transfer_from, .. } = chain.get_gas_usage_limit(coin);

	// Optimization: Calculate price denominator once.
	// Formula: coin_price * 10^(gas_coin_decimals)
	let gas_coin_decimals = chain.to_gas_coin().decimals() as i32;
	let price_denom = coin_price * 10f64.powi(gas_coin_decimals);

	// 4. Define Calculation Logic (Closure to handle repetition)
	// Returns: (Calculated Fee U256, Required allowance val)
	let calc_snapshot = |target_amt: U256| -> eyre::Result<(U256, U256)> {
		// NOTE: For C Pay, allowance is either 0 or MAX unlike in NC Pay.
		let is_suff = !allowance.is_zero() && allowance.ge(&target_amt);

		let (est_gas_usage, required_allowance_val) = if is_suff {
			(permit_transfer_from, U256::ZERO)
		} else {
			(approve + permit_transfer_from, target_amt - allowance)
		};

		// NOTE: Add platform_fee on top of network fee.
		let platform_payment_fee_multiplier = 1.15;
		// fee = (gas_usage * gas_price * gas_token_price) / (coin_price * 10^decimals)
		let est_gas_fee = (est_gas_usage * gas_price) as f64;
		let est_fee_f64 =
			est_gas_fee * gas_token_price * platform_payment_fee_multiplier / price_denom;
		let est_fee_u256 = parse_human_fmt_to_u256(&est_fee_f64.to_string(), coin_decimals, false)?;

		Ok((est_fee_u256, required_allowance_val))
	};

	// 5. Initial Calculation
	let (mut est_fee_u256, mut required_allowance_val) = calc_snapshot(tot_amount)?;

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
		tot_amount += est_fee_u256;
		if !use_in_ui && tot_amount.gt(&balance) {
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
///   ```
///   allowance >= total_spend_for_that_coin
///   ```
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
///   → resulting in different gas usage per coin
///
/// ## Arguments
/// - `chain`: selected chain
/// - `tot_amount_per_coin`: Aggregated total amount per coin across the batch
///
///   ```
///   USDT → sum(all USDT entries) USDC → sum(all USDC entries)
///   ```
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
/// This implementation assumes that `PreOcpValuesNcwBulk.est_fee` is expressed in the
/// **default stablecoin unit** (e.g., USDT).
///
/// If fee denomination needs to change (e.g., per-coin or gas token), only the final
/// aggregation/formatting step should be adjusted.
///
/// ## Design pros
/// - Handles mixed allowance states across coins
/// - Avoids incorrect fee scaling via `batch_len`. E.g. est_fee * batch_len
/// - Mirrors single-pay correctness guarantees
/// - Deterministic and stable after 2-pass computation (in case of fee-excl.)
pub fn compute_est_fee_ncw_bulkpay(
	chain: ChainName,
	tot_amount_per_coin: HashMap<StableCoin, U256>,
	params: PreOcpValuesNcwParamsBulk,
	is_fee_incl: bool,
	use_in_ui: bool,
) -> eyre::Result<PreOcpValuesNcwBulk> {
	let PreOcpValuesNcwParamsBulk { coin_entries: params_coin_entries, gas_price, gas_token_price } =
		params;

	let mut coin_entries = Vec::with_capacity(params_coin_entries.len());
	let mut total_est_fee_u256 = U256::ZERO;

	for entry in params_coin_entries {
		let coin = entry.coin;
		let coin_decimals = coin.decimals();

		let allowance = U256::from_str(&entry.allowance).wrap_err("Failed to parse allowance")?;
		let balance = parse_human_fmt_to_u256(&entry.balance, coin_decimals, false)
			.wrap_err("Failed to parse balance")?;

		let mut tot_amount = tot_amount_per_coin
			.get(&coin)
			.copied()
			.ok_or_eyre(format!("Missing total amount for coin: {coin:?}"))?;

		if !use_in_ui && tot_amount.gt(&balance) {
			return Err(UfiError::InsufficientBalance.into())
		}

		let GasEstimate { approve, permit_transfer_from, .. } = chain.get_gas_usage_limit(coin);

		// fee = (gas_usage * gas_price * gas_token_price * platform_multiplier)
		//       / (coin_price * 10^(gas_coin_decimals))
		let gas_coin_decimals = chain.to_gas_coin().decimals() as i32;
		let price_denom = entry.coin_price * 10f64.powi(gas_coin_decimals);

		let calc_snapshot = |target_amt: U256| -> eyre::Result<(U256, U256)> {
			let is_suff = !allowance.is_zero() && allowance.ge(&target_amt);

			let (est_gas_usage, required_allowance_val) = if is_suff {
				(permit_transfer_from, U256::ZERO)
			} else {
				(approve + permit_transfer_from, target_amt - allowance)
			};

			let platform_payment_fee_multiplier = 1.15;
			let est_gas_fee = (est_gas_usage * gas_price) as f64;
			let est_fee_f64 =
				est_gas_fee * gas_token_price * platform_payment_fee_multiplier / price_denom;
			let est_fee_u256 =
				parse_human_fmt_to_u256(&est_fee_f64.to_string(), coin_decimals, false)?;

			Ok((est_fee_u256, required_allowance_val))
		};

		let (mut est_fee_u256, mut required_allowance_val) = calc_snapshot(tot_amount)?;

		// Same reasoning as single-pay:
		// for fee-excl, total spend becomes amount + est_fee, which can change allowance
		// sufficiency, so recompute once using the updated spend.
		if !is_fee_incl {
			tot_amount += est_fee_u256;

			if !use_in_ui && tot_amount.gt(&balance) {
				return Err(UfiError::InsufficientBalance.into())
			}

			(est_fee_u256, required_allowance_val) = calc_snapshot(tot_amount)?;
		}

		total_est_fee_u256 = total_est_fee_u256
			.checked_add(est_fee_u256)
			.ok_or_eyre("Overflow while summing bulk-pay estimated fees")?;

		coin_entries.push(PreOcpValuesNcwBulkCoin {
			coin,
			is_suff: required_allowance_val.is_zero(),
			required_allowance: fmt_output(required_allowance_val, coin_decimals)?,
			balance: entry.balance,
		});
	}

	Ok(PreOcpValuesNcwBulk {
		coin_entries,
		est_fee: fmt_output(total_est_fee_u256, StableCoin::default().decimals())?,
	})
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
	total_spend += U256::from(10_u128) * U256::from(10).pow(U256::from(coin_decimals));

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
pub fn sanitize_and_parse_amount(amount: &str, coin: StableCoin) -> eyre::Result<U256> {
	let amount_u256 = parse_human_fmt_to_u256(amount, coin.decimals(), true)?;
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
) -> eyre::Result<()> {
	let amount_u256 = sanitize_and_parse_amount(amount, coin)?;
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

/// Test
/// ```sh
/// RUSTFLAGS="-Awarnings" cargo t -p unifi-sdk-primitives -F utils -- utils::tests --show-output
/// ```
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn req_allowance_tests() {
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
}
