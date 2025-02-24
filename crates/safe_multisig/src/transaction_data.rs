use alloy_primitives::{Address, Bytes, TxHash, U256};
use alloy_signer::k256::ecdsa::SigningKey;
use alloy_signer_local::LocalSigner;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Call = 0,
    DelegateCall = 1,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaTransactionData {
    pub to: Address,
    pub value: U256,
    pub data: String,
    pub operation: OperationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeTransactionData {
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
    pub operation: u8,
    pub safe_tx_gas: U256,
    pub base_gas: U256,
    pub gas_price: U256,
    pub gas_token: Address,
    pub refund_receiver: Address,
    pub nonce: U256,
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
    pub safe_tx: SafeTransactionData,
    pub contract_transaction_hash: TxHash,
    pub sender: Address,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}
