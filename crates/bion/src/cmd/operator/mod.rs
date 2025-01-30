use add::AddCommand;
use clap::{Parser, Subcommand};

mod add;

#[derive(Debug, Parser)]
pub struct OperatorCommand {}

#[derive(Debug, Subcommand)]
pub enum OperatorSubcommands {
    Add(AddCommand),
}
