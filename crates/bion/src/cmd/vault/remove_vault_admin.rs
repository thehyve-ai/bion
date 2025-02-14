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
        utils::get_chain_id,
        vault::{
            consts::{VAULT_DEFINITIONS_FILE, VAULT_DIRECTORY},
            utils::get_or_create_vault_admin_definitions,
        },
    },
    common::DirsCliArgs,
    utils::{print_success_message, write_to_json_file},
};

#[derive(Debug, Parser)]
pub struct RemoveVaultAdminCommand {
    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl RemoveVaultAdminCommand {
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
            "🔄 Checking if a vault admin with the provided alias is imported.".bright_cyan()
        );

        let mut vault_admins_map = get_or_create_vault_admin_definitions(chain_id, &dirs)?;
        if let Some((_, vault_admin_address)) = vault_admins_map.get_key_value(&alias) {
            let data_dir = dirs.data_dir(Some(chain_id))?;
            let vault_admins_dir = data_dir.join(VAULT_DIRECTORY);
            let vault_admin_config_dir = vault_admins_dir.join(vault_admin_address.to_string());
            let network_definitions_path = vault_admins_dir.join(VAULT_DEFINITIONS_FILE);

            // remove the network config folder
            remove_dir_all(&vault_admin_config_dir).map_err(|e| {
                eyre::eyre!(format!(
                    "Unable to remove vault admin config directory: {:?}: {:?}",
                    vault_admin_config_dir, e
                ))
            })?;

            vault_admins_map.remove(&alias);
            write_to_json_file(network_definitions_path, &vault_admins_map, false)
                .map_err(|e| eyre::eyre!(e))?;

            print_success_message(
                format!("✅ Successfully removed vault admin: {}", alias).as_str(),
            );
        } else {
            eyre::bail!("Vault admin with the provided alias is not imported.");
        }

        Ok(())
    }
}
