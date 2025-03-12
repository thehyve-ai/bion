use alloy_primitives::{Address, Bytes, TxHash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeTransactionData {
    pub to: String,
    pub value: u64,
    pub data: Bytes,
    pub operation: u8,
    pub safe_tx_gas: u64,
    pub base_gas: u64,
    pub gas_price: u64,
    pub gas_token: Address,
    pub refund_receiver: Address,
    pub nonce: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposeTransactionBody {
    #[serde(flatten)]
    pub safe_tx: SafeTransactionData,
    pub contract_transaction_hash: TxHash,
    pub sender: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}
