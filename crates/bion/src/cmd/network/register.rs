use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::ens::NameOrAddress;
use hyve_cli_runner::CliContext;

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::utils::get_chain_id,
    common::{DirsCliArgs, SigningMethod},
    symbiotic::{calls::is_network, consts::get_network_registry},
    utils::{print_error_message, print_loading_until_async, validate_cli_args},
};

use super::utils::get_network_config;

#[derive(Debug, Parser)]
pub struct RegisterCommand {
    #[arg(value_name = "ADDRESS", help = "The address to add.")]
    pub address: Address,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,

    #[clap(flatten)]
    tx: TransactionOpts,

    /// Timeout for sending the transaction.
    #[arg(long, env = "ETH_TIMEOUT")]
    pub timeout: Option<u64>,

    /// The number of confirmations until the receipt is fetched.
    #[arg(long, default_value = "1")]
    confirmations: u64,
}

impl RegisterCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            alias,
            dirs,
            mut eth,
            tx,
            confirmations,
            timeout,
        } = self;

        validate_cli_args(Some(address), &eth).await?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        let network_config = get_network_config(chain_id, alias, &dirs)?;
        if let Some(signing_method) = network_config.signing_method {
            match signing_method {
                SigningMethod::Keystore => {
                    let password = rpassword::prompt_password_stdout("\nEnter keystore password")?;
                    eth.wallet.keystore_path = Some(
                        network_config
                            .keystore_file
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    );
                    eth.wallet.keystore_password = Some(password);
                }
                SigningMethod::Ledger => {
                    eth.wallet.ledger = true;
                }
                SigningMethod::Trezor => {
                    eth.wallet.trezor = true;
                }
            }
        }

        let network_registry = get_network_registry(chain_id)?;

        let is_registered = print_loading_until_async(
            "Checking network registration status",
            is_network(address, network_registry, &provider),
        )
        .await?;
        if is_registered {
            print_error_message("Network is already registered");
            return Ok(());
        }

        let to = NameOrAddress::Address(network_registry);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("registerNetwork()".to_string()),
            args: vec![],
            cast_async: true,
            confirmations,
            command: None,
            unlocked: true,
            timeout,
            tx,
            eth,
            path: None,
        };
        arg.run().await
    }
}
