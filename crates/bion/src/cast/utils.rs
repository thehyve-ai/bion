use foundry_common::abi::{encode_function_args, get_func};

/// Returns the Etherscan transaction URL for the given chain ID and transaction hash.
///
/// # Arguments
///
/// * `chain_id` - A u64 representing the chain ID.
/// * `tx` - A String holding the transaction hash.
///
/// # Panics
///
/// Panics if the chain ID is not one of the supported ones.
pub fn etherscan_tx_url(chain_id: u64, tx: String) -> String {
    match chain_id {
        1 => format!("https://etherscan.io/tx/{}", tx),
        17000 => format!("https://holesky.etherscan.io/tx/{}", tx),
        11155111 => format!("https://sepolia.etherscan.io/tx/{}", tx),
        _ => panic!("Unsupported chain id: {}", chain_id),
    }
}

pub fn calldata_encode(sig: impl AsRef<str>, args: &[impl AsRef<str>]) -> eyre::Result<String> {
    let func = get_func(sig.as_ref())?;
    let calldata = encode_function_args(&func, args)?;
    Ok(alloy_primitives::hex::encode_prefixed(calldata))
}
