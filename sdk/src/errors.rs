// use axum_core::response::{IntoResponse, Response};
// use eyre::ErrReport;
// use http::StatusCode;
use thiserror::Error as ThisError;

#[allow(dead_code)]
#[derive(ThisError, Debug)]
pub enum OmniPayError {
	#[error(
		"API Server is Offline. \nPlease retry later or contact support if the problem persists."
	)]
	RequestToAPIServerFailed,
	#[error("Not enough params provided to fill all placeholders.")]
	LessParamsForApiPath,
	#[error("Extra params provided to fill all placeholders.")]
	MoreParamsForApiPath,
	#[error("Unclosed placeholder found in template.")]
	UnclosedPlaceholderInApiPathTemplate,
}
