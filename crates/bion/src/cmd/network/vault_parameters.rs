use alloy_primitives::{aliases::U96, Address};
use clap::Parser;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use foundry_common::provider::RetryProvider;
use hyve_cli_runner::CliContext;
use itertools::Itertools;
use prettytable::row;

use crate::{
    cmd::{
        alias_utils::get_alias_config,
        utils::{format_number_with_decimals, get_chain_id},
    },
    common::DirsCliArgs,
    symbiotic::{
        calls::{get_delegator_type, get_max_network_limit},
        consts::{get_network_registry, get_vault_factory},
        network_utils::{get_network_metadata, validate_network_symbiotic_status},
        vault_utils::{
            fetch_token_datas, fetch_vault_addresses, fetch_vault_datas,
            fetch_vault_symbiotic_metadata, get_vault_network_limit_formatted,
            validate_vault_symbiotic_status, VaultData, VaultDataTableBuilder,
        },
    },
    utils::{print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Get detailed information for a vault and your network.")]
pub struct VaultParametersCommand {
    #[arg(value_name = "VAULT", help = "The name or address of the vault.")]
    vault: String,

    #[arg(value_name = "SUBNETWORK", help = "The subnetwork index.")]
    subnetwork: U96,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl VaultParametersCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            vault,
            subnetwork,
            alias,
            dirs,
            eth,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let network_config = get_alias_config(chain_id, alias, &dirs)?;
        let network = network_config.address;
        let network_registry = get_network_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;
        let vault = get_vault_address(vault, chain_id, vault_factory, &provider).await?;

        validate_network_symbiotic_status(network, network_registry, &provider).await?;

        let vault = print_loading_until_async(
            "Fetching vault info",
            VaultData::load(chain_id, vault, true, &provider),
        )
        .await?;

        let network_metadata = print_loading_until_async(
            "Fetching network metadata",
            get_network_metadata(network_config.address),
        )
        .await?;

        let Some(delegator) = vault.delegator else {
            eyre::bail!("Invalid vault delegator.");
        };
        let delegator_type = get_delegator_type(delegator, &provider).await?;

        let max_network_limit =
            get_max_network_limit(network_config.address, subnetwork, delegator, &provider).await?;

        let mut table = VaultDataTableBuilder::from_vault_data(vault.clone())
            .with_name()
            .with_network(network, network_metadata)
            .with_subnetwork_identifier(network, subnetwork)?
            .with_delegator()
            .with_slasher()
            .with_current_epoch()
            .with_epoch_duration()
            .with_next_epoch_start()
            .with_time_till_next_epoch()
            .build();

        let max_limit_formatted =
            format_number_with_decimals(max_network_limit, vault.decimals.unwrap())?;
        let max_limit_display = if max_limit_formatted == "0.000" {
            "-".to_string()
        } else {
            format!(
                "{} ({} {})",
                max_network_limit,
                max_limit_formatted,
                vault.symbol.clone().unwrap()
            )
        };
        let vault_network_limit_display = get_vault_network_limit_formatted(
            &provider,
            network,
            subnetwork,
            &vault,
            delegator,
            delegator_type,
            max_limit_display.clone(),
        )
        .await?;

        table.add_row(row![Fcb -> "Max Network Limit", max_limit_display]);
        table.add_row(row![Fcb -> "Vault Network Limit", vault_network_limit_display]);

        table.printstd();

        Ok(())
    }
}

async fn get_vault_address(
    address_or_name: String,
    chain_id: u64,
    vault_factory: Address,
    provider: &RetryProvider,
) -> eyre::Result<Address> {
    if let Ok(vault) = Address::parse_checksummed(address_or_name.clone(), None) {
        validate_vault_symbiotic_status(vault, vault_factory, provider).await?;
        return Ok(vault);
    }

    let vault_addresses = print_loading_until_async(
        "Fetching Symbiotic vaults",
        fetch_vault_addresses(provider, chain_id),
    )
    .await?;
    let vaults = print_loading_until_async(
        "Fetching vault data",
        fetch_vault_datas(provider, chain_id, vault_addresses),
    )
    .await?;
    let vaults = print_loading_until_async(
        "Fetching vault metadata",
        fetch_vault_symbiotic_metadata(vaults),
    )
    .await?;
    let mut matches = vec![];
    for v in vaults {
        let Some(symbiotic_metadata) = v.symbiotic_metadata.clone() else {
            continue;
        };

        if symbiotic_metadata
            .name
            .to_lowercase()
            .contains(address_or_name.to_lowercase().as_str())
        {
            matches.push(v);
        }
    }

    if matches.is_empty() {
        eyre::bail!("No vaults found matching the given name.");
    } else if matches.len() == 1 {
        println!(
            "{}",
            format!(
                "Vault found with name: {} and address: {}",
                matches[0].symbiotic_metadata.clone().unwrap().name.bold(),
                matches[0].address.to_string().bold()
            )
            .bright_cyan()
        );
        return Ok(matches[0].address);
    }

    let matches = fetch_token_datas(provider, chain_id, matches).await?;

    println!("Found {} vaults matching the given name", matches.len());

    let matches = matches
        .into_iter()
        .sorted_by(|a, b| {
            a.symbiotic_metadata
                .clone()
                .unwrap()
                .name
                .cmp(&b.symbiotic_metadata.clone().unwrap().name)
        })
        .collect::<Vec<_>>();

    let mut options = matches
        .iter()
        .map(|v| {
            let active_stake_formatted = format!(
                "{} {}",
                v.active_stake_formatted().unwrap(),
                v.symbol.clone().unwrap()
            );
            format!(
                "{} ({})",
                v.symbiotic_metadata.clone().unwrap().name,
                active_stake_formatted
            )
        })
        .collect::<Vec<_>>();
    options.push("Cancel".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("\nChoose a vault:")
        .items(&options)
        .default(0)
        .interact()
        .map_err(|e| eyre::eyre!(format!("Failed to show vault selection menu: {}", e)))?;

    if selection == matches.len() {
        eyre::bail!("Operation cancelled.");
    }

    Ok(matches[selection].address)
}
