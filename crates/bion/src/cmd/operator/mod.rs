use alloy_network::AnyNetwork;
use alloy_primitives::{Address, Bytes};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use alloy_sol_types::{sol, SolCall};
use alloy_transport::Transport;
use bls::BLSCommands;
use cast::Cast;
use clap::Subcommand;
use delete::DeleteCommand;
use get::GetCommand;
use import::ImportCommand;
use list::ListCommand;
use register::RegisterCommand;

use std::{collections::HashMap, str::FromStr};

use crate::common::consts::TESTNET_ADDRESSES;

mod delete;
mod get;
mod import;
mod list;
mod register;

pub mod bls;

sol!(
    function isEntity(address entity_) public returns (bool);
    function isOptedIn(address who, address where) public returns (bool);
);

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

async fn is_operator<P, T>(address: Address, eth_client: &Cast<P, T>) -> eyre::Result<bool>
where
    T: Transport + Clone,
    P: Provider<T, AnyNetwork>,
{
    let is_entity = isEntityCall { entity_: address };
    let bytes: Bytes = is_entity.abi_encode().into();

    let tx = TransactionRequest::default()
        .to(Address::from_str(TESTNET_ADDRESSES[OP_REGISTRY_ENTITY])?)
        .input(bytes.into());
    let tx = WithOtherFields::new(tx);

    let res = eth_client.call(&tx, None, None).await?;
    let data = isEntityCall::abi_decode_returns(res.as_ref(), false)?;

    Ok(data._0)
}

async fn is_opted_in<P, T>(
    who: Address,
    r#where: Address,
    to: Address,
    eth_client: &Cast<P, T>,
) -> eyre::Result<bool>
where
    T: Transport + Clone,
    P: Provider<T, AnyNetwork>,
{
    let is_opted_in = isOptedInCall { who, r#where };
    let bytes: Bytes = is_opted_in.abi_encode().into();

    let tx = TransactionRequest::default().to(to).input(bytes.into());
    let tx = WithOtherFields::new(tx);

    let res = eth_client.call(&tx, None, None).await?;
    let data = isOptedInCall::abi_decode_returns(res.as_ref(), false)?;

    Ok(data._0)
}
