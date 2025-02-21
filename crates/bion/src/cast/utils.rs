use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use foundry_common::provider::RetryProvider;
use foundry_config::Config;

use super::{cmd::send::SendTxArgs, tx::CastTxBuilder};

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

pub async fn build_tx(
    args: SendTxArgs,
    config: &Config,
    provider: &RetryProvider,
) -> eyre::Result<WithOtherFields<TransactionRequest>> {
    let builder = CastTxBuilder::new(&provider, args.tx, &config)
        .await?
        .with_to(args.to)
        .await?
        .with_code_sig_and_args(None, args.sig, args.args)
        .await?;

    let (tx, _) = builder.build(config.sender).await?;
    Ok(tx)
}
