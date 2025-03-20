use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{opts::EthereumOpts, utils, utils::LoadConfig};
use hyve_cli_runner::CliContext;
use prettytable::{row, Table};

use std::time::Instant;

use crate::{
    cmd::utils::get_chain_id,
    symbiotic::network_utils::{
        fetch_network_addresses, fetch_network_data, fetch_network_symbiotic_metadata,
    },
    utils::{print_loading_until_async, validate_cli_args},
};

#[derive(Debug, Parser)]
#[clap(about = "Get information for all Symbiotic networks.")]
pub struct ListNetworksCommand {
    #[arg(
        long,
        value_name = "LIMIT",
        default_value = "10",
        help = "The number of networks to list."
    )]
    limit: u8,

    #[arg(long, help = "Only show verified networks.", default_value = "false")]
    verified_only: bool,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl ListNetworksCommand {
    pub async fn execute(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { limit, verified_only, eth } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;
        {
            let txt = format!("Loading networks on chain {} with a limit of {}.", chain_id, limit);
            println!("{}", txt.as_str().bright_cyan());
            println!("{}", "You can change this limit using --limit".bright_green())
        }

        let t1 = Instant::now();
        let network_addresses = print_loading_until_async(
            "Fetching network addresses",
            fetch_network_addresses(chain_id, &provider),
        )
        .await?;
        let total_networks = network_addresses.len();
        let networks = print_loading_until_async(
            "Fetching network data",
            fetch_network_data(chain_id, network_addresses, &provider),
        )
        .await?;
        let networks = print_loading_until_async(
            "Fetching Symbiotic metadata",
            fetch_network_symbiotic_metadata(networks),
        )
        .await?;

        {
            let txt = format!(
                "Loaded {} networks out of {} in {}ms",
                networks.len(),
                total_networks,
                t1.elapsed().as_millis()
            );
            println!("{}", txt.as_str().bright_green());
        }

        let mut table = Table::new();

        // table headers
        table.add_row(row![
            b -> "#",
            b -> "name",
            b -> "address",
            b -> "middleware",
        ]);

        let mut i = 0;
        for network in networks {
            let name = network.symbiotic_metadata.clone().map(|m| m.name);
            if verified_only && name.is_none() {
                continue;
            }

            let symbiotic_link = format!(
                "\x1B]8;;https://app.symbiotic.fi/network/{}\x1B\\{}\x1B]8;;\x1B\\",
                network.address,
                name.unwrap_or("Unverified".to_string())
            );
            let network_link: String = format!(
                "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
                network.address, network.address
            );

            let middleware_link = {
                let middleware = network.middleware_address.unwrap_or(Address::ZERO);
                if middleware != Address::ZERO {
                    format!(
                        "\x1B]8;;https://etherscan.io/address/{}\x1B\\{}\x1B]8;;\x1B\\",
                        middleware, middleware
                    )
                } else {
                    "-".to_string()
                }
            };

            let row = row![i + 1, symbiotic_link, network_link, middleware_link,];

            table.add_row(row);

            i += 1;
            if i >= limit {
                break;
            }
        }

        table.printstd();

        Ok(())
    }
}
