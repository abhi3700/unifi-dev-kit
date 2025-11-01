use colored::*;
use spinoff::spinners::SpinnerFrames;
use std::{io::Write, sync::LazyLock};
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
