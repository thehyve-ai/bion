use alloy_network::TxSigner;
use alloy_primitives::{hex::ToHexExt, keccak256, Address};
use alloy_rpc_types::{serde_helpers::WithOtherFields, TransactionRequest};
use alloy_signer::Signer;
use calls::get_version;
use consts::get_transaction_service_url;
use foundry_common::provider::RetryProvider;
use foundry_wallets::WalletSigner;
use serde::Serialize;
use transaction_data::ProposeTransactionBody;

pub mod calls;
pub mod transaction_data;

mod consts;
mod contracts;

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

        let (sender, tx_hash, signature) = match signer {
            WalletSigner::Local(s) => {
                let sender = s.address();
                let signed_tx = s.sign_transaction(tx).await?;
                let tx_hash = keccak256(signed_tx.as_bytes());
                let signature = s.sign_hash(&tx_hash).await?;
                (sender, tx_hash, signature)
            }
            WalletSigner::Ledger(s) => {
                let sender = s.get_address().await?;
                let signed_tx = s.sign_transaction(tx).await?;
                let tx_hash = keccak256(signed_tx.as_bytes());
                let signature = s.sign_hash(&tx_hash).await?;
                (sender, tx_hash, signature)
            }
            WalletSigner::Trezor(s) => {
                let sender = s.get_address().await?;
                let signed_tx = s.sign_transaction(tx).await?;
                let tx_hash = keccak256(signed_tx.as_bytes());
                let signature = s.sign_hash(&tx_hash).await?;
                (sender, tx_hash, signature)
            }
            _ => {
                eyre::bail!("")
            }
        };

        // Build the request body.
        let body = ProposeTransactionBody {
            safe_tx: tx,
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
