use alloy_primitives::{aliases::U96, Address};
use clap::Parser;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use crate::{
    cmd::utils::{format_number_with_decimals, get_chain_id},
    common::DirsCliArgs,
    symbiotic::{
        calls::{
            get_delegator_type, get_max_network_limit, get_network_limit, get_vault_collateral,
            get_vault_delegator, is_network, is_opted_in_vault, is_vault,
        },
        consts::{get_network_registry, get_vault_factory, get_vault_opt_in_service},
        network_utils::get_network_metadata,
        vault_utils::{fetch_token_data, get_vault_metadata},
        DelegatorType,
    },
    utils::{print_error_message, print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
pub struct NetworkParametersCommand {
    #[arg(value_name = "ADDRESS", help = "Address of the vault.")]
    vault: Address,

    #[arg(value_name = "NETWORK", help = "The network address.")]
    network: Address,

    #[arg(value_name = "SUBNETWORK", help = "The subnetwork identifier.")]
    subnetwork: U96,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl NetworkParametersCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            network,
            subnetwork,
            eth,
            ..
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let network_registry = get_network_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;
        let vault_opt_in_service = get_vault_opt_in_service(chain_id)?;

        let is_network = print_loading_until_async(
            "Checking network registration status",
            is_network(network, network_registry, &provider),
        )
        .await?;

        if !is_network {
            print_error_message("Network is not registered");
            return Ok(());
        }

        let is_vault = print_loading_until_async(
            "Checking vault status",
            is_vault(vault, vault_factory, &provider),
        )
        .await?;

        if !is_vault {
            print_error_message("Provided address is not a valid Symbiotic vault.");
            return Ok(());
        }

        let is_opted_in = print_loading_until_async(
            "Checking network opt in status in vault",
            is_opted_in_vault(network, vault, vault_opt_in_service, &provider),
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

        let mut table = Table::new();

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
        let network_metadata =
            print_loading_until_async("Fetching network metadata", get_network_metadata(network))
                .await?;
        let network_link = format!(
            "\x1B]8;;https://app.symbiotic.fi/vault/{}\x1B\\{}\x1B]8;;\x1B\\",
            network,
            network_metadata
                .map(|v| v.name)
                .unwrap_or("UNVERIFIED".to_string())
        );
        table.add_row(row![Fcb -> "Network", network_link]);
        table.add_row(row![Fcb -> "Subnetwork", subnetwork]);

        let max_network_limit =
            get_max_network_limit(network, subnetwork, delegator, &provider).await?;
        let mut max_network_limit_formatted =
            format_number_with_decimals(max_network_limit, collateral.decimals)?;
        if max_network_limit_formatted == "0.000" {
            max_network_limit_formatted = "-".to_string();
        }
        table.add_row(row![Fcb -> "Max Network Limit", format!("{} ({} {})", max_network_limit.to_string(), max_network_limit_formatted, collateral.symbol)]);

        let delegator_type = get_delegator_type(delegator, &provider).await?;
        if delegator_type == DelegatorType::OperatorNetworkSpecificDelegator {
            table.add_row(row![Fcb -> "Vault Network Limit", format!("{} ({} {})", max_network_limit.to_string(), max_network_limit_formatted, collateral.symbol)]);
        } else {
            let network_limit =
                get_network_limit(network, subnetwork, delegator, &provider).await?;
            let mut network_limit_formatted =
                format_number_with_decimals(network_limit, collateral.decimals)?;
            if network_limit_formatted == "0.000" {
                network_limit_formatted = "-".to_string();
            }
            table.add_row(row![Fcb -> "Vault Network Limit", format!("{} ({} {})", network_limit.to_string(), network_limit_formatted, collateral.symbol)]);
        }
        table.printstd();

        Ok(())
    }
}
