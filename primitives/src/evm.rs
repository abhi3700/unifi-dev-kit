use alloy_primitives::{Address, U256, hex::ToHexExt};
use alloy_sol_types::{SolCall, sol};

sol! {
	interface IERC20 {
		function approve(address spender, uint256 amount) external returns (bool);
	}
}

/// Get the calldata for approve fn as hex.
pub fn calldata_approve(spender: Address, amount: U256) -> Vec<u8> {
	// IERC20::approve(spender, amount)
	let call = IERC20::approveCall { spender, amount };
	call.abi_encode()
}

pub fn get_data_hex(data: Vec<u8>) -> String {
	format!("0x{}", data.encode_hex())
}
