use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::opts::{EthereumOpts, TransactionOpts};
use hyve_cli_runner::CliContext;

use std::str::FromStr;

use crate::{
    cast::cmd::send::SendTxArgs,
    common::consts::TESTNET_ADDRESSES,
    utils::{validate_address_with_signer, validate_cli_args},
};

const HYVE_MIDDLEWARE_ENTITY: &str = "hyve_middleware_service";

#[derive(Debug, Parser)]
#[clap(about = "Pauses an Operator in the HyveDA middleware.")]
pub struct PauseOperatorCommand {
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

impl PauseOperatorCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            tx,
            eth,
            timeout,
            confirmations,
        } = self;

        validate_cli_args(Some(address), &eth).await?;

        let hyve_middleware_address = Address::from_str(TESTNET_ADDRESSES[HYVE_MIDDLEWARE_ENTITY])?;

        let to = foundry_common::ens::NameOrAddress::Address(hyve_middleware_address);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("pauseOperator()".to_string()),
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
