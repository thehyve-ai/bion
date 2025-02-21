use alloy_primitives::{Address, TxHash};
use alloy_rpc_types::{serde_helpers::WithOtherFields, TransactionRequest};
use alloy_signer::k256::ecdsa::SigningKey;
use alloy_signer_local::LocalSigner;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum OperationType {
    Call = 0,
    DelegateCall = 1,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaTransactionData {
    pub to: Address,
    pub value: String,
    pub data: String,
    pub operation: Option<OperationType>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeTransactionData {
    pub to: Address,
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

pub struct ProposeTransactionArgs {
    pub safe_address: Address,
    pub sender: Address,
    pub data: String,
    pub signer: LocalSigner<SigningKey>,
    pub origin: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposeTransactionBody {
    #[serde(flatten)]
    pub safe_tx: WithOtherFields<TransactionRequest>,
    pub contract_transaction_hash: TxHash,
    pub sender: Address,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}
