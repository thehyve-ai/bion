use clap::Parser;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cmd::{alias_utils::get_alias_config, utils::get_chain_id},
    common::DirsCliArgs,
    symbiotic::{calls::is_network, consts::get_network_registry},
    utils::{
        print_error_message, print_loading_until_async, print_success_message, validate_cli_args,
    },
};

#[derive(Debug, Parser)]
#[clap(about = "Check the registration status of your network.")]
pub struct StatusCommand {
    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl StatusCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self { alias, dirs, eth } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let chain_id = get_chain_id(&provider).await?;
        let network_config = get_alias_config(chain_id, alias, &dirs)?;
        let network = network_config.address;
        let network_registry = get_network_registry(chain_id)?;

        let is_registered = print_loading_until_async(
            "Checking registration status",
            is_network(network, network_registry, &provider),
        )
        .await?;

        if is_registered {
            print_success_message("Network is registered");
        } else {
            print_error_message("Network is not registered");
        }
        Ok(())
    }
}
