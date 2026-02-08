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

pub fn permit2_domain_permit_to_typed_data_json(dp: &DomainPermitForSig) -> Result<String, String> {
	use serde_json::json;

	let chain_id = dp
		.domain
		.chain_id
		.ok_or_else(|| "Missing chain_id in EIP712 domain".to_string())?;
	let verifying_contract = dp
		.domain
		.verifying_contract
		.ok_or_else(|| "Missing verifying_contract in EIP712 domain".to_string())?;

	let permitted = dp
		.permit
		.permitted
		.iter()
		.map(|p| {
			json!({
				"token": format!("{:?}", p.token),
				"amount": p.amount.to_string(), // uint256 as decimal string
			})
		})
		.collect::<Vec<_>>();

	let typed_data = json!({
		"types": {
			"EIP712Domain": [
				{ "name": "name", "type": "string" },
				{ "name": "chainId", "type": "uint256" },
				{ "name": "verifyingContract", "type": "address" }
			],
			"TokenPermissions": [
				{ "name": "token", "type": "address" },
				{ "name": "amount", "type": "uint256" }
			],
			"PermitBatchTransferFrom": [
				{ "name": "permitted", "type": "TokenPermissions[]" },
				{ "name": "spender", "type": "address" },
				{ "name": "nonce", "type": "uint256" },
				{ "name": "deadline", "type": "uint256" }
			]
		},
		"primaryType": "PermitBatchTransferFrom",
		"domain": {
			"name": dp.domain.name.clone().unwrap_or(std::borrow::Cow::Borrowed("Permit2")),
			"chainId": chain_id.to_string(),
			"verifyingContract": format!("{:?}", verifying_contract),
		},
		"message": {
			"permitted": permitted,
			"spender": format!("{:?}", dp.permit.spender),
			"nonce": dp.permit.nonce.to_string(),
			"deadline": dp.permit.deadline.to_string()
		}
	});

	serde_json::to_string(&typed_data).map_err(|e| e.to_string())
}
