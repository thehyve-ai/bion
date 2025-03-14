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
        alias_utils::{get_or_create_alias_config, get_or_create_alias_definitions},
        consts::{ALIAS_CONFIG_FILE, ALIAS_DEFINITIONS_FILE, ALIAS_DIRECTORY},
        utils::{get_address_type, get_chain_id},
    },
    common::DirsCliArgs,
    utils::{
        print_loading_until_async, print_success_message, validate_cli_args, write_to_json_file,
    },
};

#[derive(Debug, Parser)]
#[clap(about = "Add an account alias.")]
pub struct AddAliasCommand {
    #[arg(value_name = "ALIAS", help = "The account alias.")]
    alias: String,

    #[arg(value_name = "ADDRESS", help = "The address to add.")]
    address: Address,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl AddAliasCommand {
    pub async fn execute(self, _cli: CliContext) -> eyre::Result<()> {
        let Self {
            address,
            alias,
            dirs,
            eth,
        } = self;

        validate_cli_args(&eth)?;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;

        let chain_id = get_chain_id(&provider).await?;

        println!("{}{}", "Adding alias: ".bright_cyan(), alias.bold());

        let alias_definitions_path = dirs
            .data_dir(Some(chain_id))?
            .join(format!("{}/{}", ALIAS_DIRECTORY, ALIAS_DEFINITIONS_FILE));
        let alias_config_dir = dirs
            .data_dir(Some(chain_id))?
            .join(format!("{}/{}", ALIAS_DIRECTORY, self.address));
        let alias_config_path = alias_config_dir.join(ALIAS_CONFIG_FILE);

        let mut alias_map = get_or_create_alias_definitions(chain_id, &dirs)?;
        let mut alias_config =
            get_or_create_alias_config(chain_id, self.address, alias.clone(), &dirs, false)?;

        // For now terminate if the alias already exists, in the future update functionality will be added
        if alias_map.contains_key(alias.as_str())
            || alias_map
                .values()
                .map(|a| a.to_string().to_lowercase())
                .contains(&address.to_string().to_lowercase())
        {
            eyre::bail!(format!(
                "\nAlias {} or address {} already exist.",
                alias, address
            ));
        }

        let address_type = print_loading_until_async(
            "Fetching address type",
            get_address_type(address, &provider),
        )
        .await?;

        println!("\n{}{:?}", "Address type: ".bright_cyan(), address_type);

        alias_config.set_address_type(address_type);
        alias_config.set_signing_method(&dirs, &provider).await?;

        write_to_json_file(alias_config_path, &alias_config, false).map_err(|e| eyre::eyre!(e))?;

        alias_map.insert(alias_config.alias.clone(), address);
        write_to_json_file(alias_definitions_path, &alias_map, false)
            .map_err(|e| eyre::eyre!(e))?;

        print_success_message(
            format!("✅ Successfully added alias: {}", alias_config.alias).as_str(),
        );

        Ok(())
    }
}
