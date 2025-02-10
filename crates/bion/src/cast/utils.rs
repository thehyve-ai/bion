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
