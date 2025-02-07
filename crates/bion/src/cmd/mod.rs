use std::collections::HashMap;

use alloy_primitives::Address;
use clap::{command, Subcommand};
use hyve::HyveCommands;
use symbiotic::SymbioticCommands;

pub mod hyve;
pub mod network;
pub mod operator;
pub mod symbiotic;
pub(crate) mod utils;
pub mod vault;
