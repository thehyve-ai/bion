use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::opts::{EthereumOpts, TransactionOpts};
use hyve_cli_runner::CliContext;

use std::str::FromStr;

use crate::{
    cast::cmd::send::SendTxArgs, common::consts::TESTNET_ADDRESSES,
    utils::validate_address_with_signer,
};

const OPT_IN_ENTITY: &str = "vault_opt_in_service";

#[derive(Debug, Parser)]
#[clap(about = "Opt in a vault part of Symbiotic.")]
pub struct OptInCommand {
    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "Address of the signer."
    )]
    address: Address,

    #[arg(
        long,
        required = true,
        value_name = "ADDRESS",
        help = "The address of the vault to opt-in."
    )]
    vault_address: Address,

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

impl OptInCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            vault_address,
            tx,
            eth,
            timeout,
            confirmations,
        } = self;

        validate_address_with_signer(address, &eth).await?;

        let opt_in_address = Address::from_str(TESTNET_ADDRESSES[OPT_IN_ENTITY])?;

        let to = foundry_common::ens::NameOrAddress::Address(opt_in_address);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("optIn(address where)".to_string()),
            args: vec![vault_address.to_string()],
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
