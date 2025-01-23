use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::opts::{EthereumOpts, TransactionOpts};
use hyve_cli_runner::CliContext;

use std::str::FromStr;

use crate::{cast::cmd::send::SendTxArgs, common::consts::TESTNET_ADDRESSES};

const HYVE_NETWORK_ENTITY: &str = "hyve_network";
const OPT_IN_ENTITY: &str = "network_opt_in_service";

#[derive(Debug, Parser)]
#[clap(about = "Opt out ofthe HyveDA network.")]
pub struct OptOutCommand {
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

// Weird error with async trait, so we just use a normal function
impl OptOutCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            tx,
            eth,
            timeout,
            confirmations,
        } = self;

        let hyve_network = Address::from_str(TESTNET_ADDRESSES[HYVE_NETWORK_ENTITY])?;
        let opt_in_address = Address::from_str(TESTNET_ADDRESSES[OPT_IN_ENTITY])?;

        let to = foundry_common::ens::NameOrAddress::Address(opt_in_address);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("optOut(address where)".to_string()),
            args: vec![hyve_network.to_string()],
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
