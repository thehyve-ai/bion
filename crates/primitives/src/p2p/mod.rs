mod errors;
mod network;
mod rpc_methods;

use crate::aliases::TransactionHash;

pub use self::rpc_methods::*;

pub use self::network::*;

pub use self::errors::*;

/// Interface for handling mempool message data. Used in various filters in pipelines in
/// `TransactionsManager` and in queries to `TransactionPool`.
pub trait HandleMempoolData {
    /// The announcement contains no entries.
    fn is_empty(&self) -> bool;

    /// Returns the number of entries.
    fn len(&self) -> usize;

    /// Retain only entries for which the hash in the entry satisfies a given predicate.
    fn retain_by_hash(&mut self, f: impl FnMut(&TransactionHash) -> bool);
}
