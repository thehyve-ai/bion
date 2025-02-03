use std::{collections::HashMap, path::PathBuf};

use add::AddCommand;
use alloy_primitives::Address;
use clap::{Parser, Subcommand};
use hyve_cli_runner::CliContext;
use register::RegisterCommand;
use remove::RemoveCommand;
use serde::{Deserialize, Serialize};

use crate::common::SigningMethod;

mod add;
pub(crate) mod consts;
mod register;
mod remove;

#[derive(Debug, Parser)]
#[clap(about = "Manage your operator.")]
pub struct OperatorCommand {
    #[arg(value_name = "ALIAS", help = "The saved operator alias.")]
    pub alias: String,

    #[command(subcommand)]
    pub command: OperatorSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum OperatorSubcommands {
    #[command(name = "register")]
    Register(RegisterCommand),

    // Import operator management
    #[command(name = "add")]
    Add(AddCommand),

    #[command(name = "remove")]
    Remove(RemoveCommand),
}

impl OperatorCommand {
    pub async fn execute(self, ctx: CliContext) -> eyre::Result<()> {
        match self.command {
            OperatorSubcommands::Register(register) => {
                register.with_alias(self.alias).execute(ctx).await
            }
            OperatorSubcommands::Add(add) => add.with_alias(self.alias).execute(ctx).await,
            OperatorSubcommands::Remove(remove) => remove.with_alias(self.alias).execute(ctx).await,
        }
    }
}

pub type ImportedOperators = HashMap<String, Address>;

#[derive(Debug, Deserialize, Serialize)]
pub struct OperatorConfig {
    pub address: Address,
    alias: String,
    signing_method: Option<SigningMethod>,
    password_enabled: bool,
    date_created: i64,
    date_updated: i64,
    keystore_file: Option<PathBuf>,
}

impl OperatorConfig {
    pub fn new(address: Address, alias: String) -> Self {
        Self {
            address,
            alias,
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
