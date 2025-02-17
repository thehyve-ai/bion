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
    cmd::{
        alias_utils::{get_alias_config, set_foundry_signing_method},
        utils::get_chain_id,
    },
    common::DirsCliArgs,
    symbiotic::{
        calls::is_opted_in_vault,
        consts::{get_operator_registry, get_vault_factory, get_vault_opt_in_service},
        operator_utils::validate_operator_status,
        vault_utils::validate_vault_status,
    },
    utils::{print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Opt in your operator to a vault.")]
pub struct OptInVaultCommand {
    #[arg(value_name = "VAULT", help = "The address of the vault.")]
    vault: Address,

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

impl OptInVaultCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            alias,
            dirs,
            mut eth,
            tx,
            confirmations,
            timeout,
            unlocked,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let operator_config = get_alias_config(chain_id, alias, &dirs)?;
        let operator = operator_config.address;
        let vault_factory = get_vault_factory(chain_id)?;
        let opt_in_service = get_vault_opt_in_service(chain_id)?;
        let operator_registry = get_operator_registry(chain_id)?;
        set_foundry_signing_method(&operator_config, &mut eth)?;

        validate_operator_status(operator, operator_registry, &provider).await?;
        validate_vault_status(vault, vault_factory, &provider).await?;

        let is_opted_in = print_loading_until_async(
            "Checking opted in status",
            is_opted_in_vault(operator, vault, opt_in_service, &provider),
        )
        .await?;

        if is_opted_in {
            return Err(eyre::eyre!("Operator is already opted in."));
        }

        let to = NameOrAddress::Address(opt_in_service);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("optIn(address where)".to_string()),
            args: vec![vault.to_string()],
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
