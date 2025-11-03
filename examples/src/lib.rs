use colored::*;
use spinoff::spinners::SpinnerFrames;
use std::{io::Write, sync::LazyLock};
use unifi_sdk_primitives::types::{
	ChainName, OcPayReceipt, StableCoin, WalletBalancesByChain, WalletBalancesByChainCoinDetails,
	WalletBalancesByCoin, WalletBalancesByCoinChainDetails,
};
use unifi_sdk_rs::Sdk;

pub static API_BASE_URL: LazyLock<&str> = LazyLock::new(|| {
	Box::leak(std::env::var("API_BASE_URL").expect("API_BASE_URL is not set").into_boxed_str())
});
pub static API_KEY: LazyLock<&str> = LazyLock::new(|| {
	Box::leak(std::env::var("API_KEY").expect("API_KEY is not set").into_boxed_str())
});

/// Init SDK
pub fn init_sdk() -> Sdk {
	dotenvy::dotenv().ok();
	Sdk::new(*API_BASE_URL, *API_KEY)
}

pub fn take_input(title: &str) -> eyre::Result<String> {
	let mut input = String::new();
	loop {
		print!("{}", title);
		std::io::stdout().flush()?;
		std::io::stdin().read_line(&mut input)?;
		if input.is_empty() {
			continue;
		}
		input = input.trim().to_owned();
		break;
	}

	Ok(input)
}

pub async fn with_spinner<F, T>(
	spinner_type: SpinnerFrames,
	loading_msg: String,
	f: F,
	loaded_msg: Option<String>,
	show_time: bool,
) -> eyre::Result<T>
where
	F: std::future::Future<Output = eyre::Result<T>>,
{
	let start = std::time::Instant::now();
	let mut spinner = spinoff::Spinner::new(spinner_type, loading_msg, spinoff::Color::Blue);

	let result = f.await;

	match loaded_msg {
		Some(msg) => spinner.stop_with_message(&msg),
		None => spinner.stop(),
	};

	if show_time {
		println!("{}", format!("Done in {:.2} s", start.elapsed().as_secs_f64()).cyan());
	}

	result
}

pub fn ask_yes_no(question: &str) -> eyre::Result<bool> {
	loop {
		let answer = take_input(&format!("{} (yes/no): ", question))?;
		match answer.to_lowercase().as_str() {
			"yes" | "y" => return Ok(true),
			"no" | "n" => return Ok(false),
			_ => println!("{}", "❌ Invalid input. Please enter 'yes' or 'no'.".red().bold()),
		}
	}
}

pub fn display_pay_receipt(receipt: OcPayReceipt) {
	println!("{}", "----------------------------------------".dimmed());
	println!("{} {}", "Receipt ID:".bright_blue().bold(), receipt.id);
	println!("{} {}", "Chain:".bright_blue().bold(), receipt.chain.to_string());

	// Show coins with amounts and addresses
	for (idx, coin) in receipt.coins.iter().enumerate() {
		let amt = receipt.amounts.get(idx).map(|s| s.as_str()).unwrap_or("");

		let addr = receipt.to_addresses.get(idx).map(|s| s.as_str()).unwrap_or("");
		println!("{} {} -> {} {}", "Coin:".cyan(), coin.to_string(), "Amount:".cyan(), amt);
		println!("{} {}", "To:".cyan(), addr);
	}

	// Show memo only if not default or if meaningful
	let memo_str: String = receipt.memo.clone().into();
	if !memo_str.is_empty() {
		println!("{} {}", "Memo:".bright_yellow(), memo_str);
	}

	// Show primary tx_hash if present
	if !receipt.tx_hash.trim().is_empty() {
		println!("{} {}", "Tx Hash:".magenta().bold(), receipt.tx_hash);
	}

	// Show tx_hashes list if non empty
	if !receipt.tx_hashes.is_empty() {
		println!("{}", "Tx Hashes:".magenta().bold());
		for h in receipt.tx_hashes {
			if !h.trim().is_empty() {
				println!("  {}", h);
			}
		}
	}

	// Status with color
	let status = match receipt.status {
		unifi_sdk_primitives::types::OcPayReceiptStatus::Completed => "Completed".green().bold(),
		unifi_sdk_primitives::types::OcPayReceiptStatus::Processing => "Processing".yellow().bold(),
		unifi_sdk_primitives::types::OcPayReceiptStatus::Failed => "Failed".red().bold(),
	};
	println!("{} {}", "Status:".bright_blue().bold(), status);

	// Time info
	if receipt.start_ts_us > 0 && receipt.end_ts_us > 0 {
		let dur_s = (receipt.end_ts_us - receipt.start_ts_us) as f64 / 1_000_000.0;
		println!("{} {:.2} {}", "Duration:".bright_blue().bold(), dur_s, "sec");
	} else if receipt.start_ts_us > 0 {
		// Convert microseconds to seconds and then to human readable datetime
		let submitted =
			std::time::UNIX_EPOCH + std::time::Duration::from_micros(receipt.start_ts_us as u64);
		let datetime: chrono::DateTime<chrono::Local> = submitted.into();
		println!("{} {}", "Submitted:".bright_blue().bold(), datetime.format("%Y-%m-%d %H:%M:%S"));
	}

	println!("{}", "----------------------------------------".dimmed());
}

pub fn print_balances_by_chain(chain: ChainName, data: &WalletBalancesByChain) {
	println!("\n{}", "================ Wallet — by Chain ================".bold().purple());
	println!("{} {}\n", "Chain:".bold(), format!("{chain:?}").cyan());

	println!("{} {}", "Total balance:".bold(), format!("${}", data.total_usd).green().bold());
	println!("{}", "----------------------------------------------".dimmed());

	for (coin, WalletBalancesByChainCoinDetails { price_usd, balance, value_usd }) in
		&data.coin_details
	{
		print_wallet_card_coin(*coin, &price_usd, &balance, &value_usd);
	}
}

pub fn print_balances_by_coin(coin: StableCoin, data: &WalletBalancesByCoin) {
	println!("\n{}", "================ Wallet — by Asset ================".bold().purple());
	println!("{} {}\n", "Asset:".bold(), format!("{coin:?}").cyan());

	println!("{} {}", "Total balance:".bold(), format!("${}", data.total_usd).green().bold());
	println!("{}", "----------------------------------------------".dimmed());

	for (chain, WalletBalancesByCoinChainDetails { balance, value_usd }) in &data.chain_details {
		print_wallet_card_chain(*chain, balance, &value_usd);
	}
}

fn print_wallet_card_coin(coin: StableCoin, price: &str, balance: &str, value: &str) {
	let icon = match coin {
		StableCoin::USDT => "🟢",
		StableCoin::USDC => "🔵",
		StableCoin::DAI => "🟡",
	};

	println!("{}", "┌────────────────────────────────────┐".bright_black());
	println!(
		"  {icon} {}   {}",
		format!("{coin:?}").bold(),
		format!("${value}").bold().truecolor(0, 200, 130) // UniFi green
	);
	println!("  {} {}", "Price:  ".bright_black(), format!("${price}"));
	println!("  {} {}", "Balance:".bright_black(), balance);
	println!();
	println!(
		"  {}      {}",
		"↗ Pay".on_truecolor(232, 50, 140).white().bold(), // UniFi magenta
		"⬇ Deposit".on_truecolor(87, 199, 182).black().bold(),
	);
	println!("{}", "└────────────────────────────────────┘".bright_black());
	println!();
}

fn print_wallet_card_chain(chain: ChainName, balance: &str, value: &str) {
	let icon = match chain {
		ChainName::Ethereum => "⬡",
		ChainName::Polygon => "🟣",
		ChainName::Sepolia => "⬡",
		ChainName::Anvil => "🛠",
		// _ => "⚙️",
	};

	println!("{}", "┌────────────────────────────────────┐".bright_black());
	println!(
		"  {icon} {}   {}",
		format!("{chain:?}").bold(),
		format!("${value}").bold().truecolor(0, 200, 130)
	);
	println!("  {} {}", "Balance:   ".bright_black(), balance);
	println!();

	println!(
		"  {}      {}",
		"↗ Pay".on_truecolor(232, 50, 140).white().bold(), // UniFi magenta
		"⬇ Deposit".on_truecolor(87, 199, 182).black().bold(),
	);

	println!("{}", "└────────────────────────────────────┘".bright_black());
	println!();
}
