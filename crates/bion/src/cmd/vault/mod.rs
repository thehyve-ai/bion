use alloy_network::AnyNetwork;
use alloy_primitives::{Address, Bytes, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use alloy_sol_types::{sol, SolCall};
use alloy_transport::Transport;
use cast::Cast;
use clap::Subcommand;
use get::GetCommand;
use list::ListCommand;
use opt_in::OptInCommand;
use opt_out::OptOutCommand;

mod get;
mod list;
mod opt_in;
mod opt_out;

sol!(
    function activeStake() returns (uint256);
    function totalStake() returns (uint256);
);

#[derive(Debug, Subcommand)]
#[clap(about = "Commands for vault general use.")]
pub enum VaultCommands {
    #[command(name = "get")]
    Get(GetCommand),

    #[command(name = "list")]
    List(ListCommand),

    #[command(name = "opt-in")]
    OptIn(OptInCommand),

    #[command(name = "opt-out")]
    OptOut(OptOutCommand),
}

pub async fn get_active_stake<P, T>(to: Address, eth_client: &Cast<P, T>) -> eyre::Result<U256>
where
    T: Transport + Clone,
    P: Provider<T, AnyNetwork>,
{
    let active_stake_call = activeStakeCall {};
    let bytes: Bytes = active_stake_call.abi_encode().into();

    let tx = TransactionRequest::default().to(to).input(bytes.into());
    let tx = WithOtherFields::new(tx);

    let res = eth_client.call(&tx, None, None).await?;
    let data = activeStakeCall::abi_decode_returns(res.as_ref(), false)?;

    Ok(data._0)
}

pub async fn get_total_stake<P, T>(to: Address, eth_client: &Cast<P, T>) -> eyre::Result<U256>
where
    T: Transport + Clone,
    P: Provider<T, AnyNetwork>,
{
    let total_stake_call = totalStakeCall {};
    let bytes: Bytes = total_stake_call.abi_encode().into();

    let tx = TransactionRequest::default().to(to).input(bytes.into());
    let tx = WithOtherFields::new(tx);

    let res = eth_client.call(&tx, None, None).await?;
    let data = totalStakeCall::abi_decode_returns(res.as_ref(), false)?;

    Ok(data._0)
}
