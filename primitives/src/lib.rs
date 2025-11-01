use serde::Deserialize;
use std::fmt::Debug;

pub mod errors;
pub mod types;
#[cfg(feature = "utils")]
pub mod utils;

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T: Debug> {
	pub status: String,
	pub data: T,
}
