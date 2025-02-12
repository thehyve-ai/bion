use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use std::fs::remove_dir_all;

use crate::{
    cmd::{
        operator::{
            consts::{OPERATOR_DEFINITIONS_FILE, OPERATOR_DIRECTORY},
            utils::get_or_create_operator_definitions,
        },
        utils::get_chain_id,
    },
    common::DirsCliArgs,
    utils::{print_success_message, write_to_json_file},
};

#[derive(Debug, Parser)]
pub struct RemoveCommand {
    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl RemoveCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self { alias, dirs, eth } = self;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;

        println!(
            "{}",
            "🔄 Checking if an operator with the provided alias is imported.".bright_cyan()
        );

        let mut operators_map = get_or_create_operator_definitions(chain_id, &dirs)?;
        if let Some((_, network_address)) = operators_map.get_key_value(&alias) {
            let data_dir = dirs.data_dir(Some(chain_id))?;
            let operators_dir = data_dir.join(OPERATOR_DIRECTORY);
            let operator_config_dir = operators_dir.join(network_address.to_string());
            let operator_definitions_path = operator_config_dir.join(OPERATOR_DEFINITIONS_FILE);

            remove_dir_all(&operator_config_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to remove operator config directory: {:?}: {:?}",
                    operator_config_dir, e
                ))
            })?;

            operators_map.remove(&alias);
            write_to_json_file(operator_definitions_path, &operators_map, false)
                .map_err(|e| eyre::eyre!(e))?;

            print_success_message(format!("✅ Successfully removed operator: {}", alias).as_str());
        } else {
            eyre::bail!("Operator with the provided alias is not imported.");
        }

        Ok(())
    }
}
