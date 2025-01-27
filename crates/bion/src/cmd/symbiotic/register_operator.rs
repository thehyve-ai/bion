use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use std::str::FromStr;

use crate::{
    cast::cmd::send::SendTxArgs, common::consts::TESTNET_ADDRESSES,
    symbiotic::calls::get_operator_registry_status, utils::validate_address_with_signer,
};

const OP_REGISTRY_ENTITY: &str = "op_registry";

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

        validate_address_with_signer(address, &eth).await?;

        let op_registry = Address::from_str(TESTNET_ADDRESSES[OP_REGISTRY_ENTITY])?;

        // Currently the config and provider are created twice when running the Cast command.
        // This is not ideal and should be refactored.
        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let is_registered = get_operator_registry_status(address, op_registry, &provider).await?;

        if is_registered {
            return Err(eyre::eyre!("Address is already registered"));
        }

        let to = foundry_common::ens::NameOrAddress::Address(op_registry);

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
