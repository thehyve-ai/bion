use std::{collections::HashMap, path::PathBuf};

use add::AddCommand;
use alloy_primitives::Address;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::common::SigningMethod;

mod add;

#[derive(Debug, Parser)]
pub struct VaultCommand {}

#[derive(Debug, Subcommand)]
pub enum VaultSubcommands {
    // Import vault management
    Add(AddCommand),
}

pub type ImportedVaults = HashMap<String, Address>;

#[derive(Debug, Deserialize, Serialize)]
pub struct VaultConfig {
    pub address: Address,
    alias: String,
    signing_method: Option<SigningMethod>,
    password_enabled: bool,
    date_created: i64,
    date_updated: i64,
    keystore_file: Option<PathBuf>,
}

impl VaultConfig {
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
