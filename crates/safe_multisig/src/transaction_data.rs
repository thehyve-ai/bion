use alloy_primitives::{Address, TxHash, B256, U256};
use alloy_rlp::{Encodable, RlpEncodable};
use alloy_rpc_types::{serde_helpers::WithOtherFields, TransactionRequest};
use alloy_signer::k256::ecdsa::SigningKey;
use alloy_signer_local::LocalSigner;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub operation: OperationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeTransactionData {
    pub to: Address,
    pub value: String,
    pub data: String,
    pub operation: OperationType,
    pub safe_tx_gas: u64,
    pub base_gas: u64,
    pub gas_price: u128,
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

#[derive(Debug, RlpEncodable)]
pub struct EIP712TypedData {
    pub domain: EIP712Domain,
    pub types: EIP712TxTypes,
    pub message: Vec<KeyValuePair>,
}

#[derive(Debug, RlpEncodable)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, RlpEncodable)]
#[rlp(trailing)]
pub struct EIP712Domain {
    pub name: Option<Cow<'static, str>>,

    /// The current major version of the signing domain. Signatures from
    /// different versions are not compatible.
    pub version: Option<Cow<'static, str>>,

    /// The EIP-155 chain ID. The user-agent should refuse signing if it does
    /// not match the currently active chain.
    pub chain_id: Option<U256>,

    /// The address of the contract that will verify the signature.
    pub verifying_contract: Option<Address>,

    pub salt: Option<B256>,
}

#[derive(Debug, RlpEncodable)]
pub struct EIP712Field {
    pub field_type: String,
    pub name: String,
}

#[derive(Debug, RlpEncodable)]
pub struct EIP712TxTypes {
    pub eip712_domain: Vec<EIP712Field>,
    pub safe_tx: Vec<EIP712Field>,
}
