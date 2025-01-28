use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cast::cmd::send::SendTxArgs,
    symbiotic::{calls::is_operator, consts::get_operator_registry},
    utils::{try_get_chain, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Register the signer as an operator in Symbiotic.")]
pub struct RegisterOperatorCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "Address of the signer."
    )]
    address: Address,

    #[clap(flatten)]
    tx: TransactionOpts,

    #[clap(flatten)]
    eth: EthereumOpts,

    /// Timeout for sending the transaction.
    #[arg(long, env = "ETH_TIMEOUT")]
    pub timeout: Option<u64>,

    /// The number of confirmations until the receipt is fetched.
    #[arg(long, default_value = "1")]
    confirmations: u64,
}

impl RegisterOperatorCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            tx,
            eth,
            timeout,
            confirmations,
            address,
        } = self;

        validate_cli_args(Some(address), &eth).await?;

        let chain = try_get_chain(&eth.etherscan)?;
        let operator_registry = get_operator_registry(chain)?;

        // Currently the config and provider are created twice when running the Cast command.
        // This is not ideal and should be refactored.
        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let is_registered = is_operator(address, operator_registry, &provider).await?;
        if is_registered {
            return Err(eyre::eyre!("Operator is already registered"));
        }

        let to = foundry_common::ens::NameOrAddress::Address(operator_registry);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("registerOperator()".to_string()),
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
