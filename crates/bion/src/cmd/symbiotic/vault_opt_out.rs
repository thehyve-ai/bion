use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::utils::get_chain_id,
    symbiotic::{
        calls::{is_opted_in_vault, is_vault},
        consts::{get_vault_factory, get_vault_opt_in_service},
    },
    utils::validate_cli_args,
};

#[derive(Debug, Parser)]
#[clap(about = "Opt out of a Symbiotic vault.")]
pub struct VaultOptOutCommand {
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

impl VaultOptOutCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            vault_address,
            tx,
            eth,
            timeout,
            confirmations,
        } = self;

        validate_cli_args(Some(address), &eth).await?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        let opt_in_service = get_vault_opt_in_service(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;

        let is_vault = is_vault(vault_address, vault_factory, &provider).await?;
        if !is_vault {
            return Err(eyre::eyre!("Address is not a vault."));
        }

        let is_opted_in =
            is_opted_in_vault(address, vault_address, opt_in_service, &provider).await?;

        if !is_opted_in {
            return Err(eyre::eyre!(
                "Cannot opt-out of vault because the operator is not yet opted-in."
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
