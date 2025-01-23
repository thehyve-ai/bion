use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::opts::{EthereumOpts, TransactionOpts};
use hyve_cli_runner::CliContext;

use std::str::FromStr;

use crate::{cast::cmd::send::SendTxArgs, common::consts::TESTNET_ADDRESSES};

const OP_REGISTRY_ENTITY: &str = "op_registry";

#[derive(Debug, Parser)]
#[clap(about = "Register the signer as an operator in Symbiotic.")]
pub struct RegisterCommand {
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

impl RegisterCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            tx,
            eth,
            timeout,
            confirmations,
        } = self;

        let op_registry_address = Address::from_str(TESTNET_ADDRESSES[OP_REGISTRY_ENTITY])?;

        let to = foundry_common::ens::NameOrAddress::Address(op_registry_address);

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
