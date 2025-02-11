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
    symbiotic::{calls::is_vault, consts::get_vault_factory, vault_utils::validate_vault_status},
    utils::{print_error_message, print_loading_until_async, validate_cli_args},
};

use super::utils::{get_vault_admin_config, set_foundry_signing_method};

#[derive(Debug, Parser)]
pub struct SetIsDepositLimitCommand {
    #[arg(value_name = "VAULT", help = "Address of the vault.")]
    vault: Address,

    #[arg(long, value_name = "IS_DEPOSIT_LIMIT", help = "Enable deposit limit.")]
    is_deposit_limit: bool,

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

impl SetIsDepositLimitCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            is_deposit_limit,
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
        let vault_factory = get_vault_factory(chain_id)?;
        let vault_admin_config = get_vault_admin_config(chain_id, alias, &dirs)?;
        set_foundry_signing_method(&vault_admin_config, &mut eth)?;

        validate_vault_status(vault, vault_factory, &provider).await?;

        let to = NameOrAddress::Address(vault);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("setIsDepositLimit(bool status)".to_string()),
            args: vec![is_deposit_limit.to_string()],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth,
            path: None,
        };

        let _ = arg.run().await?;
        Ok(())
    }
}
