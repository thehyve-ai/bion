use alloy_primitives::{aliases::U96, Address, U256};
use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::{EthereumOpts, TransactionOpts},
    utils::{self, LoadConfig},
};
use foundry_common::ens::NameOrAddress;
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use crate::{
    cast::cmd::send::SendTxArgs,
    cmd::utils::{format_number_with_decimals, get_chain_id},
    common::DirsCliArgs,
    symbiotic::{
        calls::{
            get_delegator_type, get_max_network_limit, get_network_limit, get_vault_collateral,
            get_vault_delegator, is_opted_in_vault,
        },
        consts::{get_network_registry, get_vault_factory, get_vault_opt_in_service},
        network_utils::{get_network_metadata, validate_network_status},
        vault_utils::{fetch_token_data, get_vault_metadata, validate_vault_status},
        DelegatorType,
    },
    utils::{
        print_error_message, print_loading_until_async, read_user_confirmation, validate_cli_args,
    },
};

use super::utils::{get_network_config, set_foundry_signing_method};

#[derive(Debug, Parser)]
#[clap(about = "Set a max network limit on specific vault for your network.")]
pub struct SetMaxNetworkLimitCommand {
    #[arg(value_name = "VAULT", help = "The address of the vault.")]
    pub vault: Address,

    #[arg(
        value_name = "SUBNETWORK",
        help = "The subnetwork to set the limit for."
    )]
    pub subnetwork: U96,

    #[arg(value_name = "LIMIT", help = "The limit to set.")]
    pub limit: U256,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    tx: TransactionOpts,

    #[clap(flatten)]
    eth: EthereumOpts,

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

impl SetMaxNetworkLimitCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            subnetwork,
            limit,
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
        let network_registry = get_network_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;
        let vault_opt_in_service = get_vault_opt_in_service(chain_id)?;
        let network_config = get_network_config(chain_id, alias, &dirs)?;
        set_foundry_signing_method(&network_config, &mut eth)?;

        validate_network_status(network_config.address, network_registry, &provider).await?;
        validate_vault_status(vault, vault_factory, &provider).await?;

        let is_opted_in = print_loading_until_async(
            "Checking network opt in status in vault",
            is_opted_in_vault(
                network_config.address,
                vault,
                vault_opt_in_service,
                &provider,
            ),
        )
        .await?;

        if !is_opted_in {
            print_error_message("Network is not opted in vault.");
            return Ok(());
        }

        let delegator =
            print_loading_until_async("Fetching delegator", get_vault_delegator(vault, &provider))
                .await?;

        let collateral_address = get_vault_collateral(vault, &provider).await?;
        let collateral = fetch_token_data(chain_id, collateral_address, &provider).await?;
        if collateral.is_none() {
            print_error_message("Invalid vault collateral.");
            return Ok(());
        }

        let collateral = collateral.unwrap();
        let normalized_limit = limit;
        let limit = limit * U256::from(10).pow(U256::from(collateral.decimals));

        let to = NameOrAddress::Address(delegator);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("setMaxNetworkLimit(uint96 identifier, uint256 amount)".to_string()),
            args: vec![subnetwork.to_string(), limit.to_string()],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth,
            path: None,
        };

        // Log:
        // Increasing max network limit from {x} to {y ({y_normalized})} on vault {beautify} for subnet {z}

        // Network:  0xabc (Network Name | UNVERIFIED)
        // Subnet:
        // Vault: 0xabc (Vault Name | UNVERIFIED)
        // Old limit: 100000000000000 (1000 wstETH)
        // New limit: 160000000000000 (1600 wstETH)
        // Vault Network Limit: 80000000000000 (800 wstETH)

        println!("\n{}", "Increasing max network limit".bright_cyan());

        let mut table = Table::new();

        // load network metadata
        let network_metadata = print_loading_until_async(
            "Fetching network metadata",
            get_network_metadata(network_config.address),
        )
        .await?;
        let network_link = format!(
            "\x1B]8;;https://app.symbiotic.fi/vault/{}\x1B\\{}\x1B]8;;\x1B\\",
            network_config.address,
            network_metadata
                .map(|v| v.name)
                .unwrap_or("UNVERIFIED".to_string())
        );
        table.add_row(row![Fcb -> "Network", network_link]);
        table.add_row(row![Fcb -> "Subnetwork", subnetwork]);

        // load vault metadata
        let vault_metadata =
            print_loading_until_async("Fetching vault metadata", get_vault_metadata(vault)).await?;
        let vault_link = format!(
            "\x1B]8;;https://app.symbiotic.fi/vault/{}\x1B\\{}\x1B]8;;\x1B\\",
            vault,
            vault_metadata
                .map(|v| v.name)
                .unwrap_or("UNVERIFIED".to_string())
        );
        table.add_row(row![
            Fcb -> "Vault",
            vault_link
        ]);

        let old_limit =
            get_max_network_limit(network_config.address, subnetwork, delegator, &provider).await?;
        let mut old_limit_formatted = format_number_with_decimals(old_limit, collateral.decimals)?;
        if old_limit_formatted == "0.000" {
            old_limit_formatted = "-".to_string();
        }
        table.add_row(row![Fcb -> "Old limit", format!("{} ({} {})", old_limit.to_string(), old_limit_formatted, collateral.symbol)]);
        table.add_row(row![Fcb -> "New limit", format!("{} ({} {})", limit.to_string(), normalized_limit, collateral.symbol)]);

        let delegator_type = get_delegator_type(delegator, &provider).await?;
        if delegator_type == DelegatorType::OperatorNetworkSpecificDelegator {
            table.add_row(row![Fcb -> "Vault Network Limit", format!("{} ({} {})", old_limit.to_string(), old_limit_formatted, collateral.symbol)]);
        } else {
            let network_limit =
                get_network_limit(network_config.address, subnetwork, delegator, &provider).await?;
            let mut network_limit_formatted =
                format_number_with_decimals(network_limit, collateral.decimals)?;
            if network_limit_formatted == "0.000" {
                network_limit_formatted = "-".to_string();
            }
            table.add_row(row![Fcb -> "Vault Network Limit", format!("{} ({} {})", network_limit.to_string(), network_limit_formatted, collateral.symbol)]);
        }
        table.printstd();

        println!("\n{}", "Do you wish to continue? (y/n)".bright_cyan());

        let confirmation: String = read_user_confirmation()?;
        if confirmation.trim().to_lowercase().as_str() == "n"
            || confirmation.trim().to_lowercase().as_str() == "no"
        {
            print_error_message("Exiting...");
            return Ok(());
        }

        // Todo: in vault commands: add set-network-limit network-address subnet new-limit (normalized)
        // also prompt to continue

        // network vault-parameters {subnet} 0xabc
        // Vault name
        // network name
        // Max Network Limit:
        // Network Limit:

        // vault network-parameters 0xabc {subnet}
        // Vault name
        // Network name
        // Max Network Limit:
        // Network Limit:

        let _ = arg.run().await?;
        Ok(())
    }
}
