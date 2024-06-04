use std::cmp::Ordering;
use std::collections::BTreeSet;

use starknet_mempool_types::mempool_types::ThinTransaction;
// Assumption: for the MVP only one transaction from the same contract class can be in the mempool
// at a time. When this changes, saving the transactions themselves on the queu might no longer be
// appropriate, because we'll also need to stores transactions without indexing them. For example,
// transactions with future nonces will need to be stored, and potentially indexed on block commits.
#[derive(Clone, Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct TransactionPriorityQueue(BTreeSet<PrioritizedTransaction>);

impl TransactionPriorityQueue {
    pub fn push(&mut self, tx: ThinTransaction) {
        let mempool_tx = PrioritizedTransaction(tx);
        self.insert(mempool_tx);
    }

    // TODO(gilad): remove collect
    pub fn pop_last_chunk(&mut self, n_txs: usize) -> Vec<ThinTransaction> {
        (0..n_txs).filter_map(|_| self.pop_last().map(|tx| tx.0)).collect()
    }
}

impl From<Vec<ThinTransaction>> for TransactionPriorityQueue {
    fn from(transactions: Vec<ThinTransaction>) -> Self {
        TransactionPriorityQueue(BTreeSet::from_iter(
            transactions.into_iter().map(PrioritizedTransaction),
        ))
    }
}

#[derive(Clone, Debug, derive_more::Deref, derive_more::From)]
pub struct PrioritizedTransaction(pub ThinTransaction);

/// Compare transactions based only on their tip, a uint, using the Eq trait. It ensures that two
/// tips are either exactly equal or not.
impl PartialEq for PrioritizedTransaction {
    fn eq(&self, other: &PrioritizedTransaction) -> bool {
        self.tip == other.tip
    }
}

/// Marks this struct as capable of strict equality comparisons, signaling to the compiler it
/// adheres to equality semantics.
// Note: this depends on the implementation of `PartialEq`, see its docstring.
impl Eq for PrioritizedTransaction {}

impl Ord for PrioritizedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tip.cmp(&other.tip)
    }
}

impl PartialOrd for PrioritizedTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(dead_code)]
pub enum PriorityQueueTxResult {
    Duplicate,
    Replace(ThinTransaction),
    New,
    Ignore,
}

// Assumption: there are no gaps, and the transactions are received in order.
pub struct AddressPriorityQueue(pub Vec<ThinTransaction>);

impl AddressPriorityQueue {
    pub fn push(&mut self, tx: ThinTransaction) {
        self.0.push(tx);
    }

    #[allow(dead_code)]
    pub fn top(&self) -> Option<ThinTransaction> {
        self.0.first().cloned()
    }

    pub fn pop(&mut self) -> Option<ThinTransaction> {
        Some(self.0.remove(0))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[allow(dead_code)]
    pub fn handle_tx(&mut self, new_tx: ThinTransaction) -> PriorityQueueTxResult {
        if self.0.contains(&new_tx) {
            return PriorityQueueTxResult::Duplicate;
        }

        for (index, tx) in self.0.iter_mut().enumerate() {
            if new_tx.sender_address == tx.sender_address && new_tx.nonce == tx.nonce {
                if new_tx.tip < tx.tip {
                    return PriorityQueueTxResult::Ignore;
                }
                let old_tx = self.0.remove(index);
                self.0.insert(index, new_tx);
                return PriorityQueueTxResult::Replace(old_tx);
            }
        }

        self.push(new_tx);
        PriorityQueueTxResult::New
    }
}
