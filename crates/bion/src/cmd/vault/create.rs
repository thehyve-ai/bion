use std::str::FromStr;

use alloy_primitives::{aliases::U48, Address, U256};
use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::ens::NameOrAddress;
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};
use serde::{Deserialize, Serialize};

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::utils::{format_number_with_decimals, get_chain_id},
    common::DirsCliArgs,
    symbiotic::{
        calls::{get_token_decimals, get_token_symbol},
        consts::get_vault_configurator,
        vault_utils::get_encoded_vault_configurator_params,
    },
    utils::{
        parse_duration_secs_u48, print_error_message, print_success_message,
        read_user_confirmation, validate_cli_args,
    },
};

use super::utils::{get_vault_admin_config, set_foundry_signing_method};

#[derive(Debug, Parser)]
#[clap(about = "Create a new vault with a delegator and a slasher.")]
pub struct CreateCommand {
    #[clap(flatten)]
    vault: CreateVaultCliArgs,

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

impl CreateCommand {
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
        let vault_admin_config = get_vault_admin_config(chain_id, alias, &dirs)?;
        set_foundry_signing_method(&vault_admin_config, &mut eth)?;

        let vault_configurator = get_vault_configurator(chain_id)?;
        let vault_configurator_params = get_encoded_vault_configurator_params(
            vault.version,
            vault.collateral,
            vault.burner,
            vault.epoch_duration,
            vault.deposit_whitelist,
            vault.is_deposit_limit,
            vault.deposit_limit,
            vault.delegator_index,
            vault.delegator_hook,
            vault.with_slasher,
            vault.slasher_index,
            vault.veto_duration,
            vault.resolver_set_epochs_delay,
            &vault_admin_config,
        )?;

        // print the vault table
        let mut table = Table::new();

        let collateral_decimals = get_token_decimals(vault.collateral, &provider).await?;
        let collateral_symbol = get_token_symbol(vault.collateral, &provider).await?;
        let collateral_link = format!(
            "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
            vault.collateral, collateral_symbol
        );
        table.add_row(row![Fcb -> "Collateral",   collateral_link]);

        let burner_link = format!(
            "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
            vault.burner, vault.burner
        );
        table.add_row(row![Fcb -> "Burner",  burner_link]);

        let deposit_limit = format_deposit_limit(vault.deposit_limit, collateral_decimals).unwrap();
        table.add_row(row![Fcb -> "Deposit limit",  deposit_limit]);

        let deposit_whitelist = match vault.deposit_whitelist {
            true => "✅",
            false => "❌",
        };
        table.add_row(row![Fcb -> "Deposit whitelist",  deposit_whitelist]);
        table
            .add_row(row![Fcb -> "Epoch duration",  parse_duration_secs_u48(vault.epoch_duration)]);
        table.printstd();

        println!("\n{}", "Do you wish to continue? (y/n)".bright_cyan());

        let confirmation: String = read_user_confirmation()?;
        if confirmation.trim().to_lowercase().as_str() == "n"
            || confirmation.trim().to_lowercase().as_str() == "no"
        {
            print_error_message("Exiting...");
            return Ok(());
        }

        let to = NameOrAddress::Address(vault_configurator);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("create()".to_string()),
            args: vec![vault_configurator_params],
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
            print_success_message("✅ Successfully created vault.");
        } else {
            print_error_message("❌ Failed to create vault, please try again.");
        }
        Ok(())
    }
}

#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
struct CreateVaultCliArgs {
    // Vault params
    /// Version of the vault
    /// 1 - Common
    /// 2 - Tokenized
    #[arg(long, value_name = "VERSION")]
    version: u64,

    #[arg(long, value_name = "COLLATERAL", help = "Address of the collateral.")]
    collateral: Address,

    #[arg(
        long,
        value_name = "BURNER",
        help = "Address of the deployed burner router."
    )]
    burner: Address,

    #[arg(
        long,
        value_name = "EPOCH_DURATION",
        help = "Duration of the Vault epoch in seconds."
    )]
    epoch_duration: U48,

    #[arg(
        long,
        value_name = "DEPOSIT_WHITELIST",
        help = "Enable deposit whitelisting."
    )]
    deposit_whitelist: bool,

    #[arg(long, value_name = "IS_DEPOSIT_LIMIT", help = "Enable deposit limit.")]
    is_deposit_limit: bool,

    #[arg(long, value_name = "DEPOSIT_LIMIT", help = "The deposit limit.")]
    deposit_limit: U256,

    /// Type of the Delegator
    /// 0 - NetworkRestakeDelegator
    /// 1 - FullRestakeDelegator
    /// 2 - OperatorSpecificDelegator
    /// 3 - OperatorNetworkSpecificDelegator
    #[arg(long, value_name = "DELEGATOR_INDEX")]
    delegator_index: u64,

    #[arg(
        long,
        value_name = "DELEGATOR_HOOK",
        help = "Address of the Delegator hook."
    )]
    delegator_hook: Option<Address>,

    // Slasher params
    #[arg(
        long,
        value_name = "WITH_SLASHER",
        help = "Enables the Slasher module."
    )]
    with_slasher: bool,

    #[arg(
        long,
        value_name = "SLASHER_INDEX",
        help = "Type of the Slasher. 0 - Slasher; 1 - VetoSlasher."
    )]
    slasher_index: u64,

    #[arg(long, value_name = "VETO_DURATION", help = "Veto duration in seconds.")]
    veto_duration: U48,

    #[arg(
        long,
        value_name = "RESOLVER_SET_EPOCHS_DELAY",
        help = "The number of Vault epochs needed for the resolver to be changed."
    )]
    resolver_set_epochs_delay: U256,
}

impl FromStr for CreateVaultCliArgs {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

fn format_deposit_limit(deposit_limit: U256, decimals: u8) -> Option<String> {
    if deposit_limit == U256::ZERO || decimals == 0 {
        return Some("-".to_string());
    }
    format_number_with_decimals(deposit_limit, decimals).ok()
}
