use std::{collections::HashMap, path::PathBuf};

use add::AddCommand;
use alloy_primitives::Address;
use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use serde::{Deserialize, Serialize};
use set_max_network_limit::SetMaxNetworkLimitCommand;

use crate::common::SigningMethod;

use super::utils::AddressType;

mod add;
mod set_max_network_limit;
mod utils;

#[derive(Debug, Parser)]
#[clap(about = "Manage your network.")]
pub struct NetworkCommand {
    #[arg(value_name = "NETWORK_ALIAS", help = "The saved network alias.")]
    pub network_alias: String,

    #[command(subcommand)]
    pub command: NetworkSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum NetworkSubcommands {
    #[command(name = "set-max-network-limit")]
    SetMaxNetworkLimit(SetMaxNetworkLimitCommand),

    #[command(name = "add")]
    Add(AddCommand),
}

impl NetworkCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            NetworkSubcommands::SetMaxNetworkLimit(set_max_network_limit) => {
                set_max_network_limit.execute(ctx).await
            }
            NetworkSubcommands::Add(add) => add.with_alias(self.network_alias).run(ctx).await,
        }
    }
}

pub type ImportedNetworks = HashMap<String, Address>;

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkMetadata {
    pub address: Address,
    alias: String,
    address_type: AddressType,
    signing_method: Option<SigningMethod>,
    password_enabled: bool,
    date_created: i64,
    date_updated: i64,
    keystore_file: Option<PathBuf>,
}

impl NetworkMetadata {
    pub fn new(address: Address, alias: String) -> Self {
        Self {
            address,
            alias,
            address_type: AddressType::EOA, // default value that will be overwritten
            signing_method: None,
            password_enabled: false,
            date_created: chrono::Utc::now().timestamp(),
            date_updated: chrono::Utc::now().timestamp(),
            keystore_file: None,
        }
    }

    pub fn set_alias(&mut self, alias: String) {
        self.alias = alias;
    }

    pub fn set_address_type(&mut self, address_type: AddressType) {
        self.address_type = address_type;
    }

    pub fn set_signing_method(&mut self, signing_method: Option<SigningMethod>) {
        self.signing_method = signing_method;
    }

    pub fn set_password_enabled(&mut self, password_enabled: bool) {
        self.password_enabled = password_enabled;
    }

    pub fn set_keystore_file(&mut self, keystore_file: Option<PathBuf>) {
        self.keystore_file = keystore_file;
    }
}
