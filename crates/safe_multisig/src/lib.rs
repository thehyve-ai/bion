use alloy_primitives::{
    hex::{self, ToHexExt},
    Address, U256,
};
use alloy_signer::Signer;
use calls::{
    exec_transaction, get_nonce, get_threshold, get_transaction_hash, get_version, is_owner,
};
use colored::Colorize;
use consts::get_transaction_service_url;
use foundry_common::provider::RetryProvider;
use foundry_wallets::WalletSigner;
use semver::Version;
use transaction_data::{
    ExecutableSafeTransaction, ProposeSafeTransactionBody, SafeMetaTransaction,
};
use utils::{build_safe_tx, print_loading_until_async, read_user_confirmation};

pub mod calls;
pub mod transaction_data;

mod consts;
mod contracts;
mod utils;

pub struct SafeClient {
    tx_service_url: String,
}

impl SafeClient {
    pub fn new(chain_id: u64) -> eyre::Result<Self> {
        let tx_service_url = get_transaction_service_url(chain_id)?;

        Ok(Self { tx_service_url })
    }

    pub async fn send_tx(
        &self,
        safe_address: Address,
        signer: WalletSigner,
        tx: SafeMetaTransaction,
        provider: &RetryProvider,
    ) -> eyre::Result<Option<ExecutableSafeTransaction>> {
        let threshold = get_threshold(safe_address, provider).await?;
        if threshold == U256::from(1) {
            println!("{}", "The threshold is set to 1".bright_cyan());
            println!(
                "\n{}",
                "Do you wish to review and confirm the transaction through the Safe dashboard? (y/n)"
                    .bright_cyan()
            );

            let confirmation: String = read_user_confirmation()?;
            if confirmation.trim().to_lowercase().as_str() == "y"
                || confirmation.trim().to_lowercase().as_str() == "yes"
            {
                self.propose_tx(safe_address, signer, tx, provider).await?;
                Ok(None)
            } else {
                Ok(Some(self.execute_tx(safe_address, signer, tx, provider).await?))
            }
        } else {
            self.propose_tx(safe_address, signer, tx, provider).await?;
            Ok(None)
        }
    }

    async fn execute_tx(
        &self,
        safe_address: Address,
        signer: WalletSigner,
        tx: SafeMetaTransaction,
        provider: &RetryProvider,
    ) -> eyre::Result<ExecutableSafeTransaction> {
        let sender = match &signer {
            WalletSigner::Local(s) => s.address(),
            WalletSigner::Ledger(s) => s.get_address().await?,
            WalletSigner::Trezor(s) => s.get_address().await?,
        };

        let is_owner = print_loading_until_async(
            "Verifying ownership",
            is_owner(sender, safe_address, provider),
        )
        .await?;
        if !is_owner {
            eyre::bail!("The signer {} is not an owner of the Safe at {}. Only Safe owners can execute or propose transactions. \nHint: If you're using a hardware wallet, use --mnemonic-index to select the correct account.", sender.to_checksum(None), safe_address.to_checksum(None));
        }

        let nonce = get_nonce(safe_address, provider).await?;
        let safe_tx = build_safe_tx(tx, nonce)?;
        let tx_hash = print_loading_until_async(
            "Fetching safe tx hash",
            get_transaction_hash(&safe_tx, safe_address, provider),
        )
        .await?;

        if matches!(signer, WalletSigner::Ledger(..)) {
            let signature = signer.sign_message(tx_hash.as_slice()).await?;
            let signature = signature.as_bytes().encode_hex_with_prefix();
            let mut signature_v = u8::from_str_radix(&signature[signature.len() - 2..], 16)?;
            signature_v += 4;

            let signature =
                format!("{}{}", &signature[..signature.len() - 2], format!("{:02x}", signature_v));
            // convert hex signature to bytes
            let signature = hex::decode(signature)?;
            exec_transaction(&safe_tx, signature.as_slice(), safe_address)
        } else {
            let signature = signer.sign_hash(&tx_hash).await?;

            exec_transaction(&safe_tx, signature.as_bytes().as_slice(), safe_address)
        }
    }

    async fn propose_tx(
        &self,
        safe_address: Address,
        signer: WalletSigner,
        tx: SafeMetaTransaction,
        provider: &RetryProvider,
    ) -> eyre::Result<()> {
        let safe_version: Version =
            print_loading_until_async("Fetching safe version", get_version(safe_address, provider))
                .await?
                .parse()
                .unwrap();
        if safe_version < "1.3.0".parse().unwrap() {
            eyre::bail!("Account Abstraction functionality is not available for Safes with version lower than v1.3.0");
        }

        let sender = match &signer {
            WalletSigner::Local(s) => s.address(),
            WalletSigner::Ledger(s) => s.get_address().await?,
            WalletSigner::Trezor(s) => s.get_address().await?,
        };

        let is_owner = print_loading_until_async(
            "Verifying ownership",
            is_owner(sender, safe_address, provider),
        )
        .await?;
        if !is_owner {
            eyre::bail!("The signer {} is not an owner of the Safe at {}. Only Safe owners can execute or propose transactions. \nHint: If you're using a hardware wallet, use --mnemonic-index to select the correct account.", sender.to_checksum(None), safe_address.to_checksum(None));
        }

        let nonce =
            print_loading_until_async("Fetching nonce", get_nonce(safe_address, provider)).await?;
        let safe_tx = build_safe_tx(tx, nonce)?;
        let tx_hash = print_loading_until_async(
            "Fetching tx hash",
            get_transaction_hash(&safe_tx, safe_address, provider),
        )
        .await?;

        let signature = if matches!(signer, WalletSigner::Ledger(..)) {
            let signature = signer.sign_message(tx_hash.as_slice()).await?;
            let signature = signature.as_bytes().encode_hex_with_prefix();
            let mut signature_v = u8::from_str_radix(&signature[signature.len() - 2..], 16)?;
            signature_v += 4;

            let signature =
                format!("{}{}", &signature[..signature.len() - 2], format!("{:02x}", signature_v));
            // convert hex signature to bytes
            hex::decode(signature)?
        } else {
            let signature =
                print_loading_until_async("Getting signature", signer.sign_hash(&tx_hash)).await?;
            signature.as_bytes().to_vec()
        };

        // Build the request body.
        let body = ProposeSafeTransactionBody {
            safe_tx,
            contract_transaction_hash: tx_hash,
            sender: sender.to_checksum(None),
            signature: signature.as_slice().encode_hex_with_prefix(),
            origin: None,
        };

        // Send the POST request.
        let url =
            format!("{}/v1/safes/{}/multisig-transactions/", self.tx_service_url, safe_address);
        let client = reqwest::Client::new();
        let response = print_loading_until_async(
            "Proposing transaction",
            client.post(&url).json(&body).send(),
        )
        .await?;

        // Check if the response indicates success.
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            eyre::bail!("Failed to propose transaction: {} - {}", status, text);
        } else {
            println!("{}", "Transaction proposed successfully.".green());
        }

        Ok(())
    }
}
