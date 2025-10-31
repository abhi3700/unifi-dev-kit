use errors::OmniPayError;
use reqwest::{Client as reqwestClient, Error as reqwestError, Response as reqwestResponse};
use unifi_sdk_primitives::ApiResponse;

pub mod errors;
pub mod health;
pub mod pay;
pub mod profile;
pub mod types;
pub mod wallet;

#[macro_export]
macro_rules! http_error_message {
    ($status:expr) => {
		// Simplified error handling for different status codes
        match $status {
            401 => "🔒 <b>Unauthorized!</b> 🚫 Please check your credentials. 🔑",
            403 => "🚫 <b>Forbidden!</b> 🛑 You don't have permission to access this resource. 🔐",
            404 => "🔍 <b>Not Found!</b> 😕 The requested resource doesn't exist. 📭",
            409 => "🔍 <b>Conflict!</b> 😕 The requested resource already exists. 📭",
            429 => "⏳ <b>Too Many Requests!</b> 🐢 Please slow down and try again later. 🚦",
            500 => "🚨 <b>Internal Server Error!</b> 💻 Something went wrong on our end. Please try again later. ⏳",
            501 => "🚧 <b>Not Implemented!</b> 🔧 The server doesn't support this functionality yet.",
            502 => "🌐 <b>Bad Gateway!</b> 🔗 There's an issue with the server's upstream connection.",
            503 => "🛠️ <b>Service Unavailable!</b> ⏳ The server is temporarily unable to handle the request. Please try again later. ⏳",
            504 => "⏱️ <b>Gateway Timeout!</b> 🕰️ The server didn't receive a timely response from an upstream server.",
            400..=499 => "❌ <b>Client Error!</b> 😬 Something went wrong on your end. 🖥️",
            500..=599 => "⚠️ <b>Server Error!</b> 🔧 An unexpected server error occurred. Please try again later. ⏳",
            _ => "🚨 <b>Unexpected Error!</b> 😱",
        }
    };
}

#[derive(Clone)]
pub struct Sdk {
	pub client: reqwestClient,
	pub api_base_url: String,
	pub api_key: String,
}

impl Sdk {
	pub fn new(api_base_url: &str, api_key: &str) -> Self {
		Self {
			client: reqwestClient::new(),
			api_base_url: api_base_url.to_owned(),
			api_key: api_key.to_owned(),
		}
	}

	pub(crate) fn with_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
		req.header("Authorization", format!("Bearer {}", self.api_key))
	}

	pub(crate) async fn process_response<T>(
		resp: core::result::Result<reqwestResponse, reqwestError>,
	) -> eyre::Result<T>
	where
		T: serde::de::DeserializeOwned + Default + std::fmt::Debug + 'static,
	{
		match resp {
			Ok(resp) => {
				let status = resp.status();

				if status.is_success() {
					if std::any::TypeId::of::<T>() == std::any::TypeId::of::<()>() {
						// If `T` is `()`, skip deserialization
						return Ok(T::default());
					}

					// Deserialize JSON response for `T`
					match resp.json::<ApiResponse<T>>().await {
						Ok(resp) => return Ok(resp.data),
						Err(err) => return Err(eyre::eyre!("Failed to parse response: {}", err)),
					}
				}

				// Handle error response
				let error_text =
					resp.text().await.unwrap_or_else(|_| "Failed to read error body.".to_string());
				Err(eyre::eyre!("{}", error_text))
				// return Err(eyre::eyre!("{}: {}", http_error_message!(status.as_u16()),
				// error_text));
			},
			Err(err) => {
				#[cfg(not(target_arch = "wasm32"))]
				if err.is_connect() {
					// Specific handling for connection errors

					return Err(OmniPayError::RequestToAPIServerFailed.into());
				}

				#[cfg(target_arch = "wasm32")]
				if err.to_string().contains("error sending request") {
					// Specific handling for connection errors
					return Err(OmniPayError::RequestToAPIServerFailed.into());
				}

				// General error handling for other `reqwest` errors
				Err(eyre::eyre!("Request failed: {}", err))
			},
		}
	}
}
