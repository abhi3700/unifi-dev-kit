use alloy_primitives::{Address, Signature, U256, hex};
use alloy_sol_types::SolStruct;
use eyre::{Context, OptionExt, ensure};
use k256::ecdsa::SigningKey;
use unifi_sdk_primitives::{
	permit2::DomainPermitForSig,
	types::{BulkPayItem, ChainName, Memo, PreOcpValuesNcwAllowance, StableCoin},
	utils::fmt_output,
};

pub fn args() -> (Vec<String>, bool) {
	let mut args: Vec<_> = std::env::args().skip(1).collect();
	let submit = args.last().is_some_and(|arg| arg == "--submit");
	if submit {
		args.pop();
	}
	(args, submit)
}

pub fn test_chain(value: &str) -> eyre::Result<ChainName> {
	let chain = value.parse::<ChainName>().map_err(eyre::Report::msg)?;
	ensure!(matches!(chain, ChainName::Sepolia | ChainName::Anvil), "Use Sepolia or Anvil");
	Ok(chain)
}

pub fn payer_address(value: &str) -> eyre::Result<Address> {
	let payer = value.parse::<Address>().wrap_err("Invalid payer address")?;
	ensure!(!payer.is_zero(), "Choose a nonzero payer address");
	Ok(payer)
}

pub fn payment_item(
	payer: Address,
	recipient: &str,
	amount: &str,
	coin: &str,
	memo: Memo,
) -> eyre::Result<BulkPayItem> {
	let to = recipient.parse::<Address>().wrap_err("Invalid recipient address")?;
	ensure!(!to.is_zero() && to != payer, "Choose a nonzero recipient other than the payer");
	let coin = coin.parse::<StableCoin>().map_err(eyre::Report::msg)?;
	let amount = unifi_sdk_primitives::utils::sanitize_and_parse_amount(amount, coin, true)?;
	Ok(BulkPayItem {
		coin,
		to_address: to.to_string(),
		amount: fmt_output(amount, coin.decimals())?,
		memo,
	})
}

pub fn needs_approval(
	chain: ChainName,
	coin: StableCoin,
	allowance: &PreOcpValuesNcwAllowance,
	total: U256,
) -> bool {
	if allowance.is_suff || allowance.is_max_allowance {
		return false;
	}
	// ERC-20 approve sets the entire allowance. Use the total spend in base units.
	println!("Approve {total} {coin} base units on {chain} to {}.", chain.get_permit2_sc_addr());
	true
}

pub fn test_signer(payer: Address) -> eyre::Result<SigningKey> {
	let key = std::env::var("TEST_WALLET_PRIVATE_KEY").wrap_err("Set TEST_WALLET_PRIVATE_KEY")?;
	let invalid_key = || eyre::eyre!("TEST_WALLET_PRIVATE_KEY must be a valid 32-byte wallet key");
	let bytes = hex::decode_to_array::<_, 32>(key.strip_prefix("0x").unwrap_or(&key))
		.map_err(|_| invalid_key())?;
	let signer = SigningKey::from_slice(&bytes).map_err(|_| invalid_key())?;
	ensure!(
		Address::from_private_key(&signer) == payer,
		"Test wallet key does not match the payer"
	);
	Ok(signer)
}

pub fn expected_permissions(
	amounts: &[(StableCoin, U256)],
) -> eyre::Result<(Address, Vec<(Address, U256)>)> {
	let spender = env_address("BUNDLEPAY_ADDRESS")?;
	let permissions = amounts
		.iter()
		.map(|(coin, amount)| Ok((env_address(&format!("{coin}_ADDRESS"))?, *amount)))
		.collect::<eyre::Result<Vec<_>>>()?;
	Ok((spender, permissions))
}

pub fn sign_permit(
	signer: &SigningKey,
	response: &DomainPermitForSig,
	chain: ChainName,
	spender: Address,
	permissions: &[(Address, U256)],
	deadline: u64,
) -> eyre::Result<(String, String)> {
	let DomainPermitForSig { domain, permit } = response;
	// UniFi's Anvil deployment uses the Sepolia chain ID.
	ensure!(
		domain.name.as_deref() == Some("Permit2") &&
			domain.chain_id == Some(U256::from(11_155_111)) &&
			domain.verifying_contract == Some(chain.get_permit2_sc_addr()) &&
			domain.version.is_none() &&
			domain.salt.is_none(),
		"Unexpected Permit2 domain"
	);
	ensure!(permit.spender == spender, "Unexpected BundlePay spender");
	ensure!(permit.deadline == U256::from(deadline), "Unexpected Permit2 deadline");
	ensure!(
		permit
			.permitted
			.iter()
			.map(|item| (item.token, item.amount))
			.eq(permissions.iter().copied()),
		"Permit2 token permissions differ from the payment and fee"
	);

	let digest = permit.eip712_signing_hash(domain);
	let signature = Signature::from(signer.sign_prehash_recoverable(digest.as_slice())?);
	Ok((permit.nonce.to_string(), signature.to_string()))
}

fn env_address(name: &str) -> eyre::Result<Address> {
	let address = std::env::var(name).ok().ok_or_eyre(format!("Set {name} before submitting"))?;
	let address = address.parse::<Address>().map_err(|_| eyre::eyre!("Invalid {name}"))?;
	ensure!(!address.is_zero(), "{name} must be nonzero");
	Ok(address)
}
