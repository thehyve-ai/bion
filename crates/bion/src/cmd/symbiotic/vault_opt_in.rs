use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use std::str::FromStr;

use crate::{
    cast::cmd::send::SendTxArgs,
    symbiotic::{
        calls::{get_operator_vault_opt_in_status, is_vault},
        consts::addresses,
    },
    utils::validate_address_with_signer,
};

#[derive(Debug, Parser)]
#[clap(about = "Opt in a Symbiotic vault.")]
pub struct VaultOptInCommand {
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

impl VaultOptInCommand {
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

        let vault_opt_in_service = Address::from_str(addresses::sepolia::VAULT_OPT_IN_SERVICE)?;
        let vault_factory = Address::from_str(addresses::sepolia::VAULT_FACTORY)?;

        // Currently the config and provider are created twice when running the Cast command.
        // This is not ideal and should be refactored.
        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let is_vault = is_vault(vault_address, vault_factory, &provider).await?;
        if !is_vault {
            return Err(eyre::eyre!("Address is not a vault."));
        }

        let is_opted_in = get_operator_vault_opt_in_status(
            address,
            vault_address,
            vault_opt_in_service,
            &provider,
        )
        .await?;

        if is_opted_in {
            return Err(eyre::eyre!("Address is already opted in."));
        }

        let to = foundry_common::ens::NameOrAddress::Address(vault_opt_in_service);

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
