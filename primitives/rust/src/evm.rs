use alloy_primitives::{Address, U256, hex::ToHexExt, keccak256};
use alloy_sol_types::{SolCall, sol};
use k256::{
	ecdsa::{RecoveryId, Signature, VerifyingKey},
	elliptic_curve::{scalar::IsHigh, sec1::ToEncodedPoint},
	pkcs8::DecodePublicKey,
};

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

pub fn eip191_hash(message: &str) -> [u8; 32] {
	let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
	keccak256([prefix.as_bytes(), message.as_bytes()].concat()).0
}

pub fn public_key_der_to_uncompressed(pubkey_der: &[u8]) -> eyre::Result<Vec<u8>> {
	let public_key = k256::PublicKey::from_public_key_der(pubkey_der)?;
	let encoded = public_key.to_encoded_point(false);
	Ok(encoded.as_bytes().to_vec())
}

pub fn uncompressed_pubkey_to_address(pubkey: &[u8]) -> eyre::Result<String> {
	let uncompressed = match pubkey.first().copied() {
		Some(0x04) if pubkey.len() == 65 => pubkey.to_vec(),
		Some(0x02 | 0x03) if pubkey.len() == 33 => {
			let point = k256::EncodedPoint::from_bytes(pubkey)?;
			let public_key = k256::PublicKey::from_sec1_bytes(point.as_bytes())?;
			public_key.to_encoded_point(false).as_bytes().to_vec()
		},
		_ => eyre::bail!(
			"expected secp256k1 pubkey in SEC1 format (33-byte compressed or 65-byte uncompressed)"
		),
	};

	let hash = keccak256(&uncompressed[1..]);
	let address = Address::from_slice(&hash[12..]);
	Ok(address.to_string())
}

pub fn der_signature_to_rs64(der_sig: &[u8]) -> eyre::Result<[u8; 64]> {
	let sig = Signature::from_der(der_sig)?;
	let mut out = [0u8; 64];
	out[..32].copy_from_slice(&sig.r().to_bytes());
	out[32..].copy_from_slice(&sig.s().to_bytes());
	Ok(out)
}

pub fn normalized_signature_variants(rs64: [u8; 64]) -> eyre::Result<Vec<([u8; 64], bool)>> {
	let sig = Signature::from_slice(&rs64)?;
	let mut variants = vec![(rs64, false)];

	if sig.s().is_high().into() &&
		let Some(normalized) = sig.normalize_s()
	{
		let mut norm_rs64 = [0u8; 64];
		norm_rs64[..32].copy_from_slice(&normalized.r().to_bytes());
		norm_rs64[32..].copy_from_slice(&normalized.s().to_bytes());
		variants.push((norm_rs64, true));
	}

	Ok(variants)
}

pub fn rs64_to_eth_signature_hex(
	message: &str,
	rs64: [u8; 64],
	expected_addr: &str,
) -> eyre::Result<String> {
	let digest = eip191_hash(message);

	for (candidate_rs64, was_normalized) in normalized_signature_variants(rs64)? {
		let sig = Signature::from_slice(&candidate_rs64)?;

		for recid_byte in [0u8, 1u8] {
			let effective_recid = if was_normalized { recid_byte ^ 1 } else { recid_byte };
			let recid = RecoveryId::try_from(effective_recid)?;

			if let Ok(vk) = VerifyingKey::recover_from_prehash(&digest, &sig, recid) {
				let pubkey = vk.to_encoded_point(false);
				let addr = uncompressed_pubkey_to_address(pubkey.as_bytes())?;
				if addr.eq_ignore_ascii_case(expected_addr) {
					let mut bytes = Vec::with_capacity(65);
					bytes.extend_from_slice(&candidate_rs64);
					bytes.push(27 + effective_recid);
					return Ok(format!("0x{}", hex::encode(bytes)));
				}
			}
		}
	}

	eyre::bail!("failed to determine Ethereum recovery id")
}

pub fn recover_address_from_eth_signature(
	message: &str,
	signature_hex: &str,
) -> eyre::Result<String> {
	let sig_hex = signature_hex.strip_prefix("0x").unwrap_or(signature_hex);
	let raw = hex::decode(sig_hex)?;
	if raw.len() != 65 {
		eyre::bail!("expected 65-byte Ethereum signature");
	}

	let mut rs64 = [0u8; 64];
	rs64.copy_from_slice(&raw[..64]);

	let v = raw[64];
	let recid_byte = match v {
		27 | 28 => v - 27,
		0 | 1 => v,
		_ => return Err(eyre::eyre!("invalid recovery id in signature")),
	};

	let digest = eip191_hash(message);

	for (candidate_rs64, was_normalized) in normalized_signature_variants(rs64)? {
		let effective_recid = if was_normalized { recid_byte ^ 1 } else { recid_byte };
		let sig = Signature::from_slice(&candidate_rs64)?;
		let recid = RecoveryId::try_from(effective_recid)?;
		if let Ok(vk) = VerifyingKey::recover_from_prehash(&digest, &sig, recid) {
			let pubkey = vk.to_encoded_point(false);
			return uncompressed_pubkey_to_address(pubkey.as_bytes());
		}
	}

	eyre::bail!("failed to recover address from Ethereum signature")
}

#[cfg(test)]
mod tests {
	use super::*;
	use base64::{Engine as _, engine::general_purpose::STANDARD};

	#[test]
	fn test_kms_public_key_to_address() {
		// Output of: `$ aws kms get-public-key ...``
		let public_key_b64 = "MFYwEAYHKoZIzj0CAQYFK4EEAAoDQgAEh259uE8P45UOfZ07G1PfKXzholsaE/3Y9if5wTqV1eX27oP/NDxunwOeaP1FcKy4EhZ2hZ9HA4vQYzN6nzDc6g==";

		let pubkey_der = STANDARD.decode(public_key_b64).expect("invalid base64 public key");

		let uncompressed = public_key_der_to_uncompressed(&pubkey_der)
			.expect("DER public key -> uncompressed failed");

		assert_eq!(uncompressed.len(), 65);
		assert_eq!(uncompressed[0], 0x04);

		let derived_addr =
			uncompressed_pubkey_to_address(&uncompressed).expect("pubkey -> address failed");

		let expected_addr = "0x6e09D92ACd0D4353824759C4DAC69FB2d60B9420";

		assert!(
			derived_addr.eq_ignore_ascii_case(expected_addr),
			"address mismatch: expected {}, got {}",
			expected_addr,
			derived_addr
		);
	}

	#[test]
	fn test_eth_signature_flow() {
		let message = "Connect to UniFi";

		// This must come from: `$ aws kms sign ...``
		// It should be DER signature bytes encoded as hex.
		let der_sig_hex = "3046022100bb93cac36f8b1e5416bcdba7daf2d19b07f3167b9b1877ad6fbbd4cdb96b9af7022100a428c10648843ff4836eb78508c8abcfc3485482aa0951c5247ae5344c9a6384";

		let der_sig = hex::decode(der_sig_hex).expect("invalid DER signature hex");

		let expected_addr = "0x6e09D92ACd0D4353824759C4DAC69FB2d60B9420".to_string();

		let rs64 = der_signature_to_rs64(&der_sig).expect("DER signature -> rs64 failed");

		let eth_sig = rs64_to_eth_signature_hex(message, rs64, &expected_addr)
			.expect("failed to determine recovery id");

		let recovered =
			recover_address_from_eth_signature(message, &eth_sig).expect("recover failed");

		assert!(
			recovered.eq_ignore_ascii_case(&expected_addr),
			"address mismatch: expected {}, got {}",
			expected_addr,
			recovered
		);
	}
}
