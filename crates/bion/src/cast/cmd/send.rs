use crate::cast::{
    tx::{self, CastTxBuilder},
    utils::{calldata_encode, etherscan_tx_url},
};
use alloy_network::{AnyNetwork, EthereumWallet};
use alloy_primitives::{hex::FromHex, Bytes, B256, U256};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use alloy_signer::Signer;
use alloy_transport::Transport;
use cast::Cast;
use clap::Parser;
use colored::Colorize;
use eyre::Result;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils,
    utils::LoadConfig,
};
use foundry_common::ens::NameOrAddress;
use safe_multisig::transaction_data::SafeMetaTransaction;
use std::{path::PathBuf, str::FromStr};

/// CLI arguments for `cast send`.
#[derive(Debug, Clone, Parser)]
pub struct SendTxArgs {
    /// The destination of the transaction.
    ///
    /// If not provided, you must use cast send --create.
    #[arg(value_parser = NameOrAddress::from_str)]
    pub to: Option<NameOrAddress>,

    /// The signature of the function to call.
    pub sig: Option<String>,

    /// The arguments of the function to call.
    pub args: Vec<String>,

    /// Only print the transaction hash and exit immediately.
    #[arg(id = "async", long = "async", alias = "cast-async", env = "CAST_ASYNC")]
    pub cast_async: bool,

    /// The number of confirmations until the receipt is fetched.
    #[arg(long, default_value = "1")]
    pub confirmations: u64,

    #[command(subcommand)]
    pub command: Option<SendTxSubcommands>,

    /// Send via `eth_sendTransaction using the `--from` argument or $ETH_FROM as sender
    #[arg(long, requires = "from")]
    pub unlocked: bool,

    /// Timeout for sending the transaction.
    #[arg(long, env = "ETH_TIMEOUT")]
    pub timeout: Option<u64>,

    #[command(flatten)]
    pub tx: TransactionOpts,

    #[command(flatten)]
    pub eth: EthereumOpts,

    /// The path of blob data to be sent.
    #[arg(
        long,
        value_name = "BLOB_DATA_PATH",
        conflicts_with = "legacy",
        requires = "blob",
        help_heading = "Transaction options"
    )]
    pub path: Option<PathBuf>,
}

impl TryInto<SafeMetaTransaction> for SendTxArgs {
    type Error = eyre::Error;

    fn try_into(self) -> std::result::Result<SafeMetaTransaction, Self::Error> {
        let input = calldata_encode(self.sig.clone().unwrap(), &self.args)?;
        Ok(SafeMetaTransaction {
            to: match self.to {
                Some(NameOrAddress::Address(address)) => address,
                Some(NameOrAddress::Name(_)) => {
                    return Err(eyre::eyre!("ENS is not supported"));
                }
                None => {
                    return Err(eyre::eyre!("No address provided"));
                }
            },
            input: Bytes::from_hex(input)?,
            value: self.tx.value.unwrap_or(U256::from(0)),
        })
    }
}

#[derive(Debug, Clone, Parser)]
pub enum SendTxSubcommands {
    /// Use to deploy raw contract bytecode.
    #[command(name = "--create")]
    Create {
        /// The bytecode of the contract to deploy.
        code: String,

        /// The signature of the function to call.
        sig: Option<String>,

        /// The arguments of the function to call.
        args: Vec<String>,
    },
}

impl SendTxArgs {
    #[allow(unknown_lints, dependency_on_unit_never_type_fallback)]
    pub async fn run(self) -> eyre::Result<B256> {
        let Self {
            eth,
            to,
            mut sig,
            cast_async,
            mut args,
            tx,
            confirmations,
            command,
            unlocked,
            path,
            timeout,
        } = self;

        let blob_data = if let Some(path) = path {
            Some(std::fs::read(path)?)
        } else {
            None
        };

        let code = if let Some(SendTxSubcommands::Create {
            code,
            sig: constructor_sig,
            args: constructor_args,
        }) = command
        {
            sig = constructor_sig;
            args = constructor_args;
            Some(code)
        } else {
            None
        };

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let builder = CastTxBuilder::new(&provider, tx, &config)
            .await?
            .with_to(to)
            .await?
            .with_code_sig_and_args(code, sig, args)
            .await?
            .with_blob_data(blob_data)?;

        let timeout = timeout.unwrap_or(config.transaction_timeout);

        // Case 1:
        // Default to sending via eth_sendTransaction if the --unlocked flag is passed.
        // This should be the only way this RPC method is used as it requires a local node
        // or remote RPC with unlocked accounts.
        if unlocked {
            // only check current chain id if it was specified in the config
            if let Some(config_chain) = config.chain {
                let current_chain_id = provider.get_chain_id().await?;
                let config_chain_id = config_chain.id();
                // switch chain if current chain id is not the same as the one specified in the
                // config
                if config_chain_id != current_chain_id {
                    sh_warn!("Switching to chain {}", config_chain)?;
                    provider
                        .raw_request(
                            "wallet_switchEthereumChain".into(),
                            [serde_json::json!({
                                "chainId": format!("0x{:x}", config_chain_id),
                            })],
                        )
                        .await?;
                }
            }

            let (tx, _) = builder.build(config.sender).await?;

            cast_send(provider, tx, cast_async, confirmations, timeout).await
        // Case 2:
        // An option to use a local signer was provided.
        // If we cannot successfully instantiate a local signer, then we will assume we don't have
        // enough information to sign and we must bail.
        } else {
            // Retrieve the signer, and bail if it can't be constructed.
            let signer = eth.wallet.signer().await?;
            let from = signer.address();

            tx::validate_from_address(eth.wallet.from, from)?;

            let (tx, _) = builder.build(&signer).await?;

            let wallet = EthereumWallet::from(signer);
            let provider = ProviderBuilder::<_, _, AnyNetwork>::default()
                .wallet(wallet)
                .on_provider(&provider);

            cast_send(provider, tx, cast_async, confirmations, timeout).await
        }
    }
}

async fn cast_send<P: Provider<T, AnyNetwork>, T: Transport + Clone>(
    provider: P,
    tx: WithOtherFields<TransactionRequest>,
    cast_async: bool,
    confs: u64,
    timeout: u64,
) -> Result<B256> {
    let cast = Cast::new(provider);
    let pending_tx = cast.send(tx).await?;

    let chain_id = cast.chain_id().await?;
    let tx_hash = pending_tx.inner().tx_hash();

    // Log the transaction hash
    if cast_async {
        println!(
            "{}",
            format!(
                "Transaction sent. Etherscan link {}",
                etherscan_tx_url(chain_id, format!("{tx_hash:#x}"))
            )
            .bright_cyan()
        );
    } else {
        println!(
            "{}",
            format!(
                "Transaction sent. Etherscan link {}. Waiting for {} confirmations...",
                etherscan_tx_url(chain_id, format!("{tx_hash:#x}")),
                confs
            )
            .bright_cyan()
        );
        let _ = cast
            .receipt(format!("{tx_hash:#x}"), None, confs, Some(timeout), false)
            .await?;
        println!(
            "{}",
            format!(
                "Transaction confirmed. Etherscan link {}",
                etherscan_tx_url(chain_id, format!("{tx_hash:#x}"))
            )
            .bright_cyan()
        );
    }

    Ok(*tx_hash)
}
