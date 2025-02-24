use alloy_primitives::{hex::ToHexExt, keccak256, Address, TxKind, U256};
use alloy_rpc_types::{serde_helpers::WithOtherFields, TransactionRequest};
use alloy_signer::Signer;
use calls::get_nonce;
use consts::get_transaction_service_url;
use foundry_common::provider::RetryProvider;
use foundry_wallets::WalletSigner;
use serde_json::Value;
use transaction_data::{
    EIP712Domain, EIP712TypedData, KeyValuePair, MetaTransactionData, OperationType,
    ProposeTransactionBody, SafeTransactionData,
};
use utils::get_eip712_tx_types;

pub mod calls;
pub mod transaction_data;

mod consts;
mod contracts;
mod utils;

pub struct SafeClient {
    chain_id: u64,
    tx_service_url: String,
}

impl SafeClient {
    pub fn new(chain_id: u64) -> eyre::Result<Self> {
        let tx_service_url = get_transaction_service_url(chain_id)?;

        Ok(Self {
            chain_id,
            tx_service_url,
        })
    }

    pub async fn propose_transaction(
        &self,
        safe_address: Address,
        signer: WalletSigner,
        tx: WithOtherFields<TransactionRequest>,
        provider: &RetryProvider,
    ) -> eyre::Result<()> {
        // let safe_version = get_version(safe_address, provider).await?;
        // if safe_version < "1.3.0" {
        //     eyre::bail!("Account Abstraction functionality is not available for Safes with version lower than v1.3.0");
        // }

        let data = tx.input.data.clone().unwrap().encode_hex_with_prefix();
        let meta_tx = MetaTransactionData {
            to: match tx.to.unwrap() {
                TxKind::Call(a) => a,
                _ => {
                    eyre::bail!("Invalid tx kind")
                }
            },
            value: tx.value.unwrap().to_string(),
            data,
            operation: OperationType::Call,
        };

        let nonce = get_nonce(safe_address, provider).await?;
        let safe_tx = SafeTransactionData {
            to: meta_tx.to,
            value: meta_tx.value,
            data: meta_tx.data,
            operation: meta_tx.operation,
            safe_tx_gas: tx.gas.unwrap(),
            base_gas: 0,
            gas_price: tx.gas_price.unwrap(),
            gas_token: Address::ZERO,
            refund_receiver: Address::ZERO,
            nonce,
        };

        let json_value = serde_json::to_value(&safe_tx)?;

        // Ensure the value is a JSON object and convert it into a HashMap.
        let message = match json_value {
            Value::Object(obj) => obj
                .into_iter()
                .map(|(k, v)| KeyValuePair {
                    key: k,
                    value: v.to_string(),
                })
                .collect::<_>(),
            _ => eyre::bail!("Failed to create typed message."),
        };

        let typed_data = EIP712TypedData {
            domain: EIP712Domain {
                name: None,
                version: None,
                chain_id: Some(U256::from(self.chain_id)),
                verifying_contract: Some(safe_address),
                salt: None,
            },
            types: get_eip712_tx_types(),
            message,
        };

        let typed_data_bytes = alloy_rlp::encode(&typed_data);
        let tx_hash = keccak256(typed_data_bytes.as_slice());
        let signature = signer.sign_hash(&tx_hash).await?;
        let sender = match signer {
            WalletSigner::Local(s) => s.address(),
            WalletSigner::Ledger(s) => s.get_address().await?,
            WalletSigner::Trezor(s) => s.get_address().await?,
        };

        // Build the request body.
        let body = ProposeTransactionBody {
            safe_tx,
            contract_transaction_hash: tx_hash,
            sender,
            signature: signature.as_bytes().encode_hex_with_prefix(),
            origin: None,
        };

        // Send the POST request.
        let url = format!(
            "{}/v1/safes/{}/multisig-transactions/",
            self.tx_service_url, safe_address
        );
        let client = reqwest::Client::new();
        let response = client.post(&url).json(&body).send().await?;

        // Check if the response indicates success.
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            eyre::bail!("Failed to propose transaction: {} - {}", status, text);
        }

        Ok(())
    }
}
