use thiserror::Error as ThisError;

#[allow(dead_code)]
#[derive(ThisError, Debug)]
pub enum UfiError {
	#[error("Max. {0} decimal places allowed \nPlease enter a valid amount.")]
	MaxDecimalsReached(u8),
	#[error("Insufficient balance. \nPlease deposit enough amount to proceed.")]
	InsufficientBalance,
	#[error("Amount can't be zero. \nPlease enter a valid amount to proceed.")]
	ZeroAmount,
}
