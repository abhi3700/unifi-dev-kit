use alloy_sol_types::{Eip712Domain, sol};
use serde::{Deserialize, Serialize};

// Codegen from artifact.
sol!(
	#[allow(missing_docs)]
	#[sol(rpc)]
	BundlePayV2,
	"abi/BundlePayV2.json"
);

// NOTE: The permit struct that has to be signed is different from the contract input struct
// even though they have the same name.
// Also note that the EIP712 hash of this struct is sensitive to the order of the fields.
sol! {
	#[derive(Debug, Default, Serialize, Deserialize)]
	struct TokenPermissions {
		address token;
		uint256 amount;
	}

	#[derive(Debug, Default, Serialize, Deserialize)]
	struct PermitBatchTransferFrom {
		TokenPermissions[] permitted;
		address spender;
		uint256 nonce;
		uint256 deadline;
	}
}

impl From<PermitBatchTransferFrom> for ISignatureTransfer::PermitBatchTransferFrom {
	fn from(val: PermitBatchTransferFrom) -> Self {
		Self {
			permitted: val
				.permitted
				.into_iter()
				.map(|p| ISignatureTransfer::TokenPermissions { token: p.token, amount: p.amount })
				.collect(),
			nonce: val.nonce,
			deadline: val.deadline,
		}
	}
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct DomainPermitForSig {
	pub domain: Eip712Domain,
	pub permit: PermitBatchTransferFrom,
}
