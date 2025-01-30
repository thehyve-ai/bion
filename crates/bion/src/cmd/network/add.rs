use alloy_primitives::Address;
use clap::Parser;
use foundry_cli::utils;
use foundry_cli::{opts::EthereumOpts, utils::LoadConfig};
use hyve_cli_runner::CliContext;

use crate::cmd::utils::get_address_type;

// implementation:
// 1. bion network add <address>
// 2. check if the address is a contract
// 3. get metadata from the symbiotic info.json
// Getting network information... -> Network is <contract|EOA>
// Network is known as <info.name>. Do you want to use this name? (y/n)
// > n
// 4.a prompt name
// > y
// 4.b continue

// 5. check if the address is a registered network
// Getting network status...
// Network is <active|inactive>
// You can register the network with bion network <name> register
//

// 6. If the network is an EOA (not a contract): prompt: do you want to save the private key in a file? (y/n)
// 6.a no
// 6.a Prompt: What is your preferred signing method?
// 6.a A select menu with the options: Ledger, Keyfile, Mnemonic, Raw private key, whatever options cast has

// 6.b y -> prompt for the private key
// 6.b prompt: Do you want to create a password for the keystore file? (y/n)
// 6.b prompt: Enter a password for the keystore file

// 7
// Store the network in the config file in the datadir
// Update a network_definitions.yaml file in the datadir
// both of these you can define yourself
// network_definitions.json is an index of all the networks you have added, with structure:
// <name>: <address>
// <name>: <address>
// <name>: <address>
// ...
//
// config fiile is stored as <datadir>/<address>/config.json
// if you're storing the private key in a file, you can store it in the same directory as the config file, but as a keystore file

// config.json structure:
// {
//     "name": <name>,
//     "address": <address>,
//     "type": EOA|Multisig,
//     "signing_method": Ledger|Keyfile|Mnemonic|Raw private key|or whatever options cast has,
//     "password_enabled": true or false,
//     "date created":
//     "date updated":
//     "keystore_file": <path to the keystore file>,
//
// }

#[derive(Debug, Parser)]
#[clap(about = "Add a network to your bion config.")]
pub struct AddCommand {
    #[arg(value_name = "ADDRESS", help = "The address to add.")]
    pub address: Address,

    #[clap(flatten)]
    eth: EthereumOpts,
}

impl AddCommand {
    pub async fn run(self, _ctx: CliContext) -> eyre::Result<()> {
        let Self { address, eth } = self;

        let config = eth.load_config()?;
        let provider = utils::get_provider(&config)?;
        let address_type = get_address_type(address, &provider).await?;

        println!("Address type: {:?}", address_type);
        Ok(())
    }
}
