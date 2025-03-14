use std::fs::remove_dir_all;

use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;

use crate::{
    cmd::{
        alias_utils::get_or_create_alias_definitions,
        consts::{ALIAS_DEFINITIONS_FILE, ALIAS_DIRECTORY},
        utils::get_chain_id,
    },
    common::DirsCliArgs,
    utils::{print_success_message, validate_cli_args, write_to_json_file},
};

#[derive(Debug, Parser)]
#[clap(about = "Remove an account alias.")]
pub struct RemoveAliasCommand {
    #[arg(value_name = "ALIAS", help = "The saved alias.")]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl RemoveAliasCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self { alias, dirs, eth } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;

        println!("{}", "🔄 Checking if alias is imported.".bright_cyan());

        let mut alias_map = get_or_create_alias_definitions(chain_id, &dirs)?;
        if let Some((_, address)) = alias_map.get_key_value(&alias) {
            let data_dir = dirs.data_dir(Some(chain_id))?;
            let alias_dir = data_dir.join(ALIAS_DIRECTORY);
            let alias_config_dir = alias_dir.join(address.to_string());
            let alias_definitions_path = alias_dir.join(ALIAS_DEFINITIONS_FILE);

            remove_dir_all(&alias_config_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Failed to remove alias config directory: {:?}: {:?}",
                    alias_config_dir, e
                ))
            })?;

            alias_map.remove(&alias);
            write_to_json_file(alias_definitions_path, &alias_map, false)
                .map_err(|e| eyre::eyre!(e))?;

            print_success_message(format!("✅ Successfully removed alias: {}", alias).as_str());
        } else {
            eyre::bail!("Alias is not imported.");
        }

        Ok(())
    }
}
