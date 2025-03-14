use alloy_primitives::{aliases::U96, Address};
use clap::Parser;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cmd::{alias_utils::get_alias_config, utils::get_chain_id},
    common::DirsCliArgs,
    symbiotic::{
        calls::{
            get_delegator_type, get_max_network_limit, get_network_limit,
            get_operator_network_limit, get_operator_network_shares, get_operator_stake,
            get_total_operator_network_shares,
        },
        consts::{get_network_registry, get_operator_registry, get_vault_factory},
        network_utils::{get_network_metadata, validate_network_symbiotic_status},
        operator_utils::validate_operator_symbiotic_status,
        vault_utils::{
            validate_vault_symbiotic_status, RowPrefix, VaultData, VaultDataTableBuilder,
        },
        DelegatorType,
    },
    utils::{print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Get detailed information for a vault and your operator.")]
pub struct VaultParametersCommand {
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

impl VaultParametersCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self { vault, network, subnetwork, alias, dirs, eth } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let operator_config = get_alias_config(chain_id, alias, &dirs)?;
        let operator = operator_config.address;
        let network_registry = get_network_registry(chain_id)?;
        let operator_registry = get_operator_registry(chain_id)?;
        let vault_factory = get_vault_factory(chain_id)?;

        validate_operator_symbiotic_status(operator, operator_registry, &provider).await?;
        validate_network_symbiotic_status(network, network_registry, &provider).await?;
        validate_vault_symbiotic_status(vault, vault_factory, &provider).await?;

        let vault = print_loading_until_async(
            "Fetching vault info",
            VaultData::load(chain_id, vault, true, &provider),
        )
        .await?;

        let network_metadata =
            print_loading_until_async("Fetching network metadata", get_network_metadata(network))
                .await?;

        let Some(delegator) = vault.delegator else {
            eyre::bail!("Invalid vault delegator.");
        };

        let delegator_type = print_loading_until_async(
            "Fetching delegator type",
            get_delegator_type(delegator, &provider),
        )
        .await?;

        let max_network_limit = print_loading_until_async(
            "Fetching max network limit",
            get_max_network_limit(network, subnetwork, delegator, &provider),
        )
        .await?;

        let network_limit = print_loading_until_async(
            "Fetching network limit",
            get_network_limit(network, subnetwork, delegator, &provider),
        )
        .await?;

        let table_builder = VaultDataTableBuilder::from_vault_data(vault)
            .with_name()
            .with_network(network, network_metadata)
            .with_subnetwork_identifier(network, subnetwork)?
            .with_delegator()
            .with_slasher()
            .with_current_epoch()
            .with_epoch_duration()
            .with_next_epoch_start()
            .with_time_till_next_epoch()
            .with_max_network_limit(max_network_limit, RowPrefix::Default)?
            .with_network_limit(network_limit, RowPrefix::Default)?;

        let table_builder = match delegator_type {
            DelegatorType::NetworkRestakeDelegator => {
                let operator_network_shares = print_loading_until_async(
                    "Fetching operator network shares",
                    get_operator_network_shares(
                        network, subnetwork, operator, delegator, &provider,
                    ),
                )
                .await?;

                let total_operator_network_shares = print_loading_until_async(
                    "Fetching total operator network shares",
                    get_total_operator_network_shares(network, subnetwork, delegator, &provider),
                )
                .await?;

                table_builder
                    .with_operator_network_shares(operator_network_shares, RowPrefix::Default)?
                    .with_total_operator_network_shares(total_operator_network_shares)?
            }
            DelegatorType::FullRestakeDelegator => {
                let operator_network_limit = print_loading_until_async(
                    "Fetching operator network limit",
                    get_operator_network_limit(network, subnetwork, operator, delegator, &provider),
                )
                .await?;

                table_builder
                    .with_operator_network_limit(operator_network_limit, RowPrefix::Default)?
            }
            _ => table_builder,
        };

        let operator_stake = print_loading_until_async(
            "Fetching operator stake",
            get_operator_stake(network, subnetwork, operator, delegator, &provider),
        )
        .await?;

        let table = table_builder.with_operator_stake(operator_stake)?.build();
        table.printstd();

        Ok(())
    }
}
