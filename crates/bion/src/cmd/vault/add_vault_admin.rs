use alloy_primitives::Address;
use clap::Parser;
use colored::Colorize;
use foundry_cli::{
    opts::EthereumOpts,
    utils::{self, LoadConfig},
};
use hyve_cli_runner::CliContext;
use itertools::Itertools;

use crate::{
    cmd::{
        utils::{get_address_type, get_chain_id},
        vault::{
            consts::{VAULT_CONFIG_FILE, VAULT_DEFINITIONS_FILE, VAULT_DIRECTORY},
            utils::{get_or_create_vault_admin_config, get_or_create_vault_admin_definitions},
        },
    },
    common::DirsCliArgs,
    utils::{
        print_error_message, print_loading_until_async, print_success_message, write_to_json_file,
    },
};

#[derive(Debug, Parser)]
pub struct AddVaultAdminCommand {
    #[arg(value_name = "ADDRESS", help = "The address to add.")]
    pub address: Address,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl AddVaultAdminCommand {
    pub fn with_alias(self, alias: String) -> Self {
        Self { alias, ..self }
    }

    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            alias,
            dirs,
            eth,
        } = self;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;

        println!("{}{}", "Adding operator: ".bright_cyan(), alias.bold());

        let operator_definitions_path = dirs
            .data_dir(Some(chain_id))?
            .join(format!("{}/{}", VAULT_DIRECTORY, VAULT_DEFINITIONS_FILE));
        let operator_config_dir = dirs
            .data_dir(Some(chain_id))?
            .join(format!("{}/{}", VAULT_DIRECTORY, self.address));
        let operator_config_path = operator_config_dir.join(VAULT_CONFIG_FILE);

        let mut vault_admins_map = get_or_create_vault_admin_definitions(chain_id, &dirs)?;
        let mut vault_admin_config =
            get_or_create_vault_admin_config(chain_id, self.address, alias.clone(), &dirs)?;

        let address_type = print_loading_until_async(
            "Fetching address type",
            get_address_type(address, &provider),
        )
        .await?;

        println!("\n{}{:?}", "Address type: ".bright_cyan(), address_type);

        vault_admin_config.set_address_type(address_type);

        // For now terminate if the alias already exists, in the future update functionality will be added
        if vault_admins_map.contains_key(alias.as_str())
            || vault_admins_map
                .values()
                .into_iter()
                .map(|a| a.to_string().to_lowercase())
                .contains(&address.to_string().to_lowercase())
        {
            print_error_message(
                format!("\nVault admin with alias {} already exists.", alias).as_str(),
            );
            return Ok(());
        }

        vault_admin_config.set_signing_method(&dirs)?;

        write_to_json_file(operator_config_path, &vault_admin_config, false)
            .map_err(|e| eyre::eyre!(e))?;

        vault_admins_map.insert(vault_admin_config.alias.clone(), address);
        write_to_json_file(operator_definitions_path, &vault_admins_map, false)
            .map_err(|e| eyre::eyre!(e))?;

        print_success_message(
            format!(
                "✅ Successfully added vault admin: {}",
                vault_admin_config.alias
            )
            .as_str(),
        );

        Ok(())
    }
}
