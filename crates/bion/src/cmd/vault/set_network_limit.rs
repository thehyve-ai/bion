use alloy_primitives::{aliases::U96, hex::ToHexExt, Address, U256};
use alloy_sol_types::SolValue;
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
    cmd::{
        utils::{format_number_with_decimals, get_chain_id},
        vault::utils::{get_vault_admin_config, set_foundry_signing_method},
    },
    common::DirsCliArgs,
    symbiotic::{
        calls::{
            get_delegator_type, get_max_network_limit, get_network_limit, get_vault_collateral,
            get_vault_delegator, is_network, is_opted_in_vault, is_vault,
        },
        consts::{get_network_registry, get_vault_factory, get_vault_opt_in_service},
        network_utils::get_network_metadata,
        utils::get_subnetwork,
        vault_utils::{fetch_token_data, get_vault_metadata},
        DelegatorType,
    },
    utils::{
        print_error_message, print_loading_until_async, read_user_confirmation, validate_cli_args,
    },
};

#[derive(Debug, Parser)]
pub struct SetNetworkLimitCommand {
    #[arg(value_name = "ADDRESS", help = "The address of the network.")]
    network: Address,

    #[arg(
        value_name = "SUBNETWORK",
        help = "The subnetwork to set the limit for."
    )]
    subnetwork: U96,

    #[arg(value_name = "VAULT", help = "The address of the vault.")]
    vault: Address,

    #[arg(value_name = "LIMIT", help = "The limit to set.")]
    limit: U256,

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

impl SetNetworkLimitCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            network,
            subnetwork,
            vault,
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
        let vault_admin_config = get_vault_admin_config(chain_id, alias, &dirs)?;
        set_foundry_signing_method(&vault_admin_config, &mut eth)?;

        let subnetwork_address = get_subnetwork(network, subnetwork)?;

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
        let normalized_limit = limit;
        let limit = limit * U256::from(10).pow(U256::from(collateral.decimals));

        let to = NameOrAddress::Address(delegator);

        let arg = SendTxArgs {
            to: Some(to),
            sig: Some("setNetworkLimit(bytes32 subnetwork, uint256 amount)".to_string()),
            args: vec![
                subnetwork_address
                    .abi_encode()
                    .encode_hex_upper_with_prefix(),
                limit.to_string(),
            ],
            cast_async: false,
            confirmations,
            command: None,
            unlocked,
            timeout,
            tx,
            eth,
            path: None,
        };

        println!("\n{}", "Increasing network limit".bright_cyan());

        let mut table = Table::new();

        // load network metadata
        let network_metadata = get_network_metadata(network).await?;
        let network_link = format!(
            "\x1B]8;;https://app.symbiotic.fi/vault/{}\x1B\\{}\x1B]8;;\x1B\\",
            network,
            network_metadata
                .map(|v| v.name)
                .unwrap_or("UNVERIFIED".to_string())
        );
        table.add_row(row![Fcb -> "Network", network_link]);
        table.add_row(row![Fcb -> "Subnetwork", subnetwork]);

        // load vault metadata
        let vault_metadata = get_vault_metadata(vault).await?;
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

        let max_network_limit =
            get_max_network_limit(network, subnetwork, delegator, &provider).await?;
        let mut max_network_limit_formatted =
            format_number_with_decimals(max_network_limit, collateral.decimals)?;
        if max_network_limit_formatted == "0.000" {
            max_network_limit_formatted = "-".to_string();
        }
        table.add_row(row![Fcb -> "Max network limit", format!("{} ({} {})", max_network_limit.to_string(), max_network_limit_formatted, collateral.symbol)]);

        let delegator_type = get_delegator_type(delegator, &provider).await?;
        if delegator_type == DelegatorType::OperatorNetworkSpecificDelegator {
            print_error_message(
                "Unable to set network limit for operator network specific delegator.",
            );
            return Ok(());
        } else {
            let old_network_limit =
                get_network_limit(network, subnetwork, delegator, &provider).await?;
            if old_network_limit == limit {
                print_error_message("New limit is the same as the old limit.");
                return Ok(());
            }

            let mut old_network_limit_formatted =
                format_number_with_decimals(old_network_limit, collateral.decimals)?;
            if old_network_limit_formatted == "0.000" {
                old_network_limit_formatted = "-".to_string();
            }
            table.add_row(row![Fcb -> "Old Network Limit", format!("{} ({} {})", old_network_limit.to_string(), old_network_limit_formatted, collateral.symbol)]);
            table.add_row(row![Fcb -> "New Network Limit", format!("{} ({} {})", limit.to_string(), normalized_limit, collateral.symbol)]);
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

        let _ = arg.run().await?;
        Ok(())
    }
}
