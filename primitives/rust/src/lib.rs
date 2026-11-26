use serde::Deserialize;
use std::fmt::Debug;

pub mod errors;
#[cfg(feature = "evm")]
pub mod evm;
/// TODO: put inside evm module
#[cfg(feature = "permit2")]
pub mod permit2;
pub mod types;
#[cfg(feature = "utils")]
pub mod utils;

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T: Debug> {
	pub status: String,
	pub data: T,
}
