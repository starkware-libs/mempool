use std::cmp::Ordering;
use std::collections::BTreeSet;

use starknet_api::transaction::{Tip, TransactionHash};
use starknet_mempool_types::mempool_types::ThinTransaction;
// Assumption: for the MVP only one transaction from the same contract class can be in the mempool
// at a time. When this changes, saving the transactions themselves on the queu might no longer be
// appropriate, because we'll also need to stores transactions without indexing them. For example,
// transactions with future nonces will need to be stored, and potentially indexed on block commits.
#[derive(Clone, Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct TransactionPriorityQueue(BTreeSet<ThinPriorityTransaction>);

impl TransactionPriorityQueue {
    pub fn push(&mut self, tx: ThinPriorityTransaction) {
        self.insert(tx);
    }

    // TODO(gilad): remove collect
    pub fn pop_last_chunk(&mut self, n_txs: usize) -> Vec<ThinPriorityTransaction> {
        (0..n_txs).filter_map(|_| self.pop_last()).collect()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ThinPriorityTransaction {
    pub tx_hash: TransactionHash,
    pub tip: Tip,
}

/// Compare transactions based only on their tip, a uint, using the Eq trait. It ensures that two
/// tips are either exactly equal or not.
impl PartialEq for ThinPriorityTransaction {
    fn eq(&self, other: &ThinPriorityTransaction) -> bool {
        self.tip == other.tip
    }
}

/// Marks this struct as capable of strict equality comparisons, signaling to the compiler it
/// adheres to equality semantics.
// Note: this depends on the implementation of `PartialEq`, see its docstring.
impl Eq for ThinPriorityTransaction {}

impl Ord for ThinPriorityTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tip.cmp(&other.tip)
    }
}

impl PartialOrd for ThinPriorityTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<ThinTransaction> for ThinPriorityTransaction {
    fn from(tx: ThinTransaction) -> Self {
        ThinPriorityTransaction { tx_hash: tx.tx_hash, tip: tx.tip }
    }
}
