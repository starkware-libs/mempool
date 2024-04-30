#[cfg(test)]
#[path = "priority_queue_test.rs"]
pub mod priority_queue_test;

use starknet_api::internal_transaction::InternalTransaction;
use std::{cmp::Ordering, collections::BTreeSet};
// Assumption: for the MVP only one transaction from the same contract class can be in the mempool
// at a time. When this changes, saving the transactions themselves on the queu might no longer be
// appropriate, because we'll also need to stores transactions without indexing them. For example,
// transactions with future nonces will need to be stored, and potentially indexed on block commits.
#[derive(Clone, Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct TransactionPriorityQueue(BTreeSet<PQTransaction>);

impl TransactionPriorityQueue {
    pub fn push(&mut self, tx: InternalTransaction) {
        let mempool_tx = PQTransaction(tx);
        self.insert(mempool_tx);
    }

    // Removes and returns the transaction with the highest tip.
    pub fn pop(&mut self) -> Option<InternalTransaction> {
        self.pop_last().map(|tx| tx.0)
    }
}

impl From<Vec<InternalTransaction>> for TransactionPriorityQueue {
    fn from(transactions: Vec<InternalTransaction>) -> Self {
        TransactionPriorityQueue(BTreeSet::from_iter(
            transactions.into_iter().map(PQTransaction),
        ))
    }
}

#[derive(Clone, Debug, derive_more::Deref, derive_more::From)]
pub struct PQTransaction(pub InternalTransaction);

// Compare transactions based on their tip only, which implies `Eq`, because `tip` is uint.
impl PartialEq for PQTransaction {
    fn eq(&self, other: &PQTransaction) -> bool {
        self.tip() == other.tip()
    }
}

/// Marks this struct as capable of strict equality comparisons, signaling to the compiler it
/// adheres to equality semantics.
// Note: this depends on the implementation of `PartialEq`, see its docstring.
impl Eq for PQTransaction {}

impl Ord for PQTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tip().cmp(&other.tip())
    }
}

impl PartialOrd for PQTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
