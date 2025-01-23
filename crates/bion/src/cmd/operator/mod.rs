use alloy_primitives::Address;
use bls::BLSCommands;
use clap::Subcommand;
use delete::DeleteCommand;
use get::GetCommand;
use import::ImportCommand;
use list::ListCommand;
use register::RegisterCommand;

use std::collections::HashMap;

mod delete;
mod get;
mod import;
mod list;
mod register;

pub mod bls;

const OP_REGISTRY_ENTITY: &str = "op_registry";
const IMPORTED_ADDRESSES_FILE: &str = "imported-addresses.json";
const IMPORTED_ADDRESSES_DIR: &str = "state";

#[derive(Debug, Subcommand)]
#[clap(about = "Manage your operator account and keys.")]
pub enum OperatorCommands {
    #[command(name = "bls", subcommand)]
    BLS(BLSCommands),

    #[command(name = "delete")]
    Delete(DeleteCommand),

    #[command(name = "get")]
    Get(GetCommand),

    #[command(name = "import")]
    Import(ImportCommand),

    #[command(name = "list")]
    List(ListCommand),

    #[command(name = "register")]
    Register(RegisterCommand),
}

pub type ImportedAddresses = HashMap<Address, Option<String>>;
