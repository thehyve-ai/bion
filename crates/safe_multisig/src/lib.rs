use alloy_primitives::{hex::ToHexExt, Address, TxKind, U256};
use alloy_rpc_types::{serde_helpers::WithOtherFields, TransactionRequest};
use alloy_signer::Signer;
use calls::{get_nonce, get_transaction_hash, get_version};
use consts::get_transaction_service_url;
use foundry_common::provider::RetryProvider;
use foundry_wallets::WalletSigner;
use semver::Version;
use transaction_data::{ProposeTransactionBody, SafeTransactionData};

pub mod calls;
pub mod transaction_data;

mod consts;
mod contracts;

pub struct SafeClient {
    tx_service_url: String,
}

impl SafeClient {
    pub fn new(chain_id: u64) -> eyre::Result<Self> {
        let tx_service_url = get_transaction_service_url(chain_id)?;

        Ok(Self { tx_service_url })
    }

    pub async fn propose_transaction(
        &self,
        safe_address: Address,
        signer: WalletSigner,
        tx: WithOtherFields<TransactionRequest>,
        provider: &RetryProvider,
    ) -> eyre::Result<()> {
        let safe_version: Version = get_version(safe_address, provider).await?.parse().unwrap();
        if safe_version < "1.3.0".parse().unwrap() {
            eyre::bail!("Account Abstraction functionality is not available for Safes with version lower than v1.3.0");
        }

        let data = tx.input.data.clone().unwrap();
        let nonce = get_nonce(safe_address, provider).await?;
        let safe_tx = SafeTransactionData {
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
        };

        let tx_hash = get_transaction_hash(
            safe_tx.to.clone(),
            safe_tx.value.clone(),
            safe_tx.data.clone(),
            safe_tx.operation,
            safe_tx.safe_tx_gas,
            safe_tx.base_gas,
            safe_tx.gas_price,
            Address::ZERO,
            Address::ZERO,
            safe_tx.nonce,
            safe_address,
            provider,
        )
        .await?;

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
