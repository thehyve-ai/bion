use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum OperationType {
    Call = 0,
    DelegateCall = 1,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeTransactionData {
    pub to: String,
    pub value: String,
    pub data: String,
    pub operation: OperationType,
    pub safe_tx_gas: String,
    pub base_gas: String,
    pub gas_price: String,
    pub gas_token: String,
    pub refund_receiver: String,
    pub nonce: u64,
}

impl SafeTransactionData {
    pub fn new(
        to: String,
        value: String,
        data: String,
        operation: OperationType,
        safe_tx_gas: String,
        base_gas: String,
        gas_price: String,
        gas_token: String,
        refund_receiver: String,
        nonce: u64,
    ) -> Self {
        Self {
            to,
            value,
            data,
            operation,
            safe_tx_gas,
            base_gas,
            gas_price,
            gas_token,
            refund_receiver,
            nonce,
        }
    }
}

pub struct ProposeTransactionArgs {
    pub safe_address: String,
    pub safe_transaction_data: SafeTransactionData,
    pub safe_tx_hash: String,
    pub sender_address: String,
    pub sender_signature: String,
    pub origin: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposeTransactionBody {
    #[serde(flatten)]
    pub safe_tx: SafeTransactionData,
    pub contract_transaction_hash: String,
    pub sender: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}
