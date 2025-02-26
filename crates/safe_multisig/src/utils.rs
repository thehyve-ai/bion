use alloy_primitives::{Address, Bytes, TxKind, U256};
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;

use crate::transaction_data::SafeTransactionData;

pub fn build_safe_tx(
    data: Bytes,
    tx: WithOtherFields<TransactionRequest>,
    nonce: U256,
) -> eyre::Result<SafeTransactionData> {
    Ok(SafeTransactionData {
        to: match tx.to.unwrap() {
            TxKind::Call(a) => a,
            _ => {
                eyre::bail!("Invalid tx kind")
            }
        },
        value: tx.value.unwrap_or_else(|| U256::from(0)),
        data,
        operation: 0,
        safe_tx_gas: U256::from(tx.gas.unwrap_or(0)),
        base_gas: U256::from(0),
        gas_price: U256::from(0),
        gas_token: Address::ZERO,
        refund_receiver: Address::ZERO,
        nonce,
    })
}
