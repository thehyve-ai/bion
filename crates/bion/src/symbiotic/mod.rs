pub mod calls;
pub mod consts;
pub mod contracts;
pub mod network_utils;
pub mod utils;
pub mod vault_utils;

#[derive(Debug, PartialEq)]
pub enum DelegatorType {
    NetworkRestakeDelegator = 0,
    FullRestakeDelegator = 1,
    OperatorSpecificDelegator = 2,
    OperatorNetworkSpecificDelegator = 3,
}

impl From<u64> for DelegatorType {
    fn from(value: u64) -> Self {
        match value {
            0 => DelegatorType::NetworkRestakeDelegator,
            1 => DelegatorType::FullRestakeDelegator,
            2 => DelegatorType::OperatorSpecificDelegator,
            3 => DelegatorType::OperatorNetworkSpecificDelegator,
            _ => panic!("Invalid DelegatorType"),
        }
    }
}
