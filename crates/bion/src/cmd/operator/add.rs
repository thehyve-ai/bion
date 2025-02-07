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
        operator::{
            consts::{OPERATOR_CONFIG_FILE, OPERATOR_DEFINITIONS_FILE, OPERATOR_DIRECTORY},
            utils::{get_or_create_operator_config, get_or_create_operator_definitions},
        },
        utils::{get_address_type, get_chain_id, AddressType},
    },
    common::DirsCliArgs,
    symbiotic::{calls::is_operator, consts::get_operator_registry},
    utils::{
        print_error_message, print_loading_until_async, print_success_message, write_to_json_file,
    },
};

#[derive(Debug, Parser)]
pub struct AddCommand {
    #[arg(value_name = "ADDRESS", help = "The address to add.")]
    pub address: Address,

    #[arg(skip)]
    alias: String,

    #[clap(flatten)]
    dirs: DirsCliArgs,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl AddCommand {
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
        let operator_registry = get_operator_registry(chain_id)?;

        println!("{}{}", "Adding operator: ".bright_cyan(), alias.bold());

        let operator_definitions_path = dirs.data_dir(Some(chain_id))?.join(format!(
            "{}/{}",
            OPERATOR_DIRECTORY, OPERATOR_DEFINITIONS_FILE
        ));
        let operator_config_dir = dirs
            .data_dir(Some(chain_id))?
            .join(format!("{}/{}", OPERATOR_DIRECTORY, self.address));
        let operator_config_path = operator_config_dir.join(OPERATOR_CONFIG_FILE);

        let mut operators_map = get_or_create_operator_definitions(chain_id, &dirs)?;
        let mut operator_config =
            get_or_create_operator_config(chain_id, self.address, alias.clone(), &dirs)?;

        let address_type = print_loading_until_async(
            "Fetching address type",
            get_address_type(address, &provider),
        )
        .await?;

        println!("\n{}{:?}", "Address type: ".bright_cyan(), address_type);

        operator_config.set_address_type(address_type);

        // if address_type == AddressType::Contract {
        //     print_error_message("Address is a contract.");
        //     return Ok(());
        // }

        // For now terminate if the alias already exists, in the future update functionality will be added
        if operators_map.contains_key(alias.as_str())
            || operators_map
                .values()
                .into_iter()
                .map(|a| a.to_string().to_lowercase())
                .contains(&address.to_string().to_lowercase())
        {
            print_error_message(
                format!("\nOperator with alias {} already exists.", alias).as_str(),
            );
            return Ok(());
        }

        let is_operator = print_loading_until_async(
            "Checking operator status",
            is_operator(address, operator_registry, &provider),
        )
        .await?;

        if is_operator {
            println!("\n{}", "Operator is registered".bright_cyan());
        } else {
            println!(
                "\n{}",
                format!("Operator is not registered, you can register the operator with `bion operator {} register`", operator_config.alias)
                    .bright_cyan()
            );
        }

        operator_config.set_signing_method(&dirs)?;

        write_to_json_file(operator_config_path, &operator_config, false)
            .map_err(|e| eyre::eyre!(e))?;

        operators_map.insert(operator_config.alias.clone(), address);
        write_to_json_file(operator_definitions_path, &operators_map, false)
            .map_err(|e| eyre::eyre!(e))?;

        print_success_message(
            format!("✅ Successfully added operator: {}", operator_config.alias).as_str(),
        );

        Ok(())
    }
}
