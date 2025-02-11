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
    common::DirsCliArgs,
    symbiotic::{calls::is_operator, consts::get_operator_registry},
    utils::{print_error_message, print_loading_until_async, validate_cli_args},
};

use super::utils::{get_operator_config, set_foundry_signing_method};

#[derive(Debug, Parser)]
pub struct RegisterCommand {
    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,

    #[clap(flatten)]
    tx: TransactionOpts,

    /// Send via `eth_sendTransaction using the `--from` argument or $ETH_FROM as sender
    #[arg(long, requires = "from")]
    pub unlocked: bool,

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
            alias,
            dirs,
            mut eth,
            tx,
            confirmations,
            timeout,
            unlocked,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let operator_registry = get_operator_registry(chain_id)?;
        let operator_config = get_operator_config(chain_id, alias, &dirs)?;
        set_foundry_signing_method(&operator_config, &mut eth)?;

        let is_registered = print_loading_until_async(
            "Checking registration status",
            is_operator(operator_config.address, operator_registry, &provider),
        )
        .await?;

        if is_registered {
            print_error_message("Operator is already registered");
            return Ok(());
        }

        let to = NameOrAddress::Address(operator_registry);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("registerOperator()".to_string()),
            args: vec![],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth,
            path: None,
        };

        let _ = arg.run().await?;
        Ok(())
    }
}
