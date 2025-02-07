use alloy_primitives::Address;
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
    symbiotic::{
        calls::{is_delegator, is_vault},
        consts::{get_delegator_factory, get_vault_factory},
    },
    utils::{
        print_error_message, print_loading_until_async, print_success_message, validate_cli_args,
    },
};

use super::utils::{get_vault_admin_config, set_foundry_signing_method};

#[derive(Debug, Parser)]
pub struct SetDelegatorCommand {
    #[arg(value_name = "VAULT", help = "Address of the vault.")]
    vault: Address,

    #[arg(value_name = "DELEGATOR", help = "Address of the delegator.")]
    delegator: Address,

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

impl SetDelegatorCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            delegator,
            alias,
            dirs,
            mut eth,
            tx,
            unlocked,
            timeout,
            confirmations,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        let vault_admin_config = get_vault_admin_config(chain_id, alias, &dirs)?;
        set_foundry_signing_method(&vault_admin_config, &mut eth)?;

        let delegator_factory = get_delegator_factory(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;

        let is_vault = print_loading_until_async(
            "Checking vault status",
            is_vault(vault, vault_factory, &provider),
        )
        .await?;

        if !is_vault {
            print_error_message("Provided address is not a valid Symbiotic vault.");
            return Ok(());
        }

        let is_delegator = print_loading_until_async(
            "Checking delegator status",
            is_delegator(delegator, delegator_factory, &provider),
        )
        .await?;

        if !is_delegator {
            print_error_message("Provided address is not a valid Symbiotic delegator.");
            return Ok(());
        }

        let to = NameOrAddress::Address(vault);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("setDelegator(address delegator)".to_string()),
            args: vec![delegator.to_string()],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth,
            path: None,
        };

        if let Ok(..) = arg.run().await {
            print_success_message("✅ Successfully set vault delegator.");
        } else {
            print_error_message("❌ Failed to set vault delegator, please try again.");
        }
        Ok(())
    }
}
