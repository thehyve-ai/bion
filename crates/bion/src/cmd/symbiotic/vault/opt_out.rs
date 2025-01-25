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
    symbiotic::calls::get_operator_vault_opt_in_status, utils::validate_address_with_signer,
};

const OPT_IN_ENTITY: &str = "vault_opt_in_service";

#[derive(Debug, Parser)]
#[clap(about = "Opt out of a vault part of Symbiotic.")]
pub struct OptOutCommand {
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

impl OptOutCommand {
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

        let opt_in_service = Address::from_str(TESTNET_ADDRESSES[OPT_IN_ENTITY])?;

        // Currently the config and provider are created twice when running the Cast command.
        // This is not ideal and should be refactored.
        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let is_opted_in =
            get_operator_vault_opt_in_status(address, vault_address, opt_in_service, &provider)
                .await?;

        if !is_opted_in {
            return Err(eyre::eyre!(
                "Cannot opt-out of vault because the address is not yet opted-in."
            ));
        }

        let to = foundry_common::ens::NameOrAddress::Address(opt_in_service);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("optOut(address where)".to_string()),
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
