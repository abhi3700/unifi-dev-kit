use serde::Deserialize;
use std::fmt::Debug;

pub mod types;

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T: Debug> {
	pub status: String,
	pub data: T,
}
