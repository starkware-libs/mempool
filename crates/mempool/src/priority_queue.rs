use std::cmp::Ordering;
use std::collections::{BTreeSet, VecDeque};

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::{Tip, TransactionHash};
use starknet_mempool_types::mempool_types::ThinTransaction;
// Assumption: for the MVP only one transaction from the same contract class can be in the mempool
// at a time. When this changes, saving the transactions themselves on the queu might no longer be
// appropriate, because we'll also need to stores transactions without indexing them. For example,
// transactions with future nonces will need to be stored, and potentially indexed on block commits.

#[derive(Clone, Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct TransactionQueue(BTreeSet<QueuedTransaction>);

impl TransactionQueue {
    /// Adds a transaction to the mempool, ensuring unique keys.
    /// Panics: if given a duplicate tx.
    pub fn push(&mut self, tx: QueuedTransaction) {
        assert!(self.insert(tx), "Keys should be unique; duplicates are checked prior.");
    }

    // TODO(gilad): remove collect
    pub fn pop_last_chunk(&mut self, n_txs: usize) -> Vec<TransactionHash> {
        (0..n_txs).filter_map(|_| self.pop_last().map(|tx| tx.tx_hash)).collect()
    }
}

// TODO: remove allow dead code when `address_to_nonce` is added.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct QueuedTransaction {
    pub sender_address: ContractAddress,
    pub nonce: Nonce,
    pub tx_hash: TransactionHash,
    pub tip: Tip,
}

/// Compare transactions based only on their tip, a uint, using the Eq trait. It ensures that two
/// tips are either exactly equal or not.
impl PartialEq for QueuedTransaction {
    fn eq(&self, other: &QueuedTransaction) -> bool {
        self.tip == other.tip && self.tx_hash == other.tx_hash
    }
}

/// Marks this struct as capable of strict equality comparisons, signaling to the compiler it
/// adheres to equality semantics.
// Note: this depends on the implementation of `PartialEq`, see its docstring.
impl Eq for QueuedTransaction {}

impl Ord for QueuedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tip.cmp(&other.tip).then_with(|| self.tx_hash.cmp(&other.tx_hash))
    }
}

impl PartialOrd for QueuedTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<&ThinTransaction> for QueuedTransaction {
    fn from(tx: &ThinTransaction) -> Self {
        QueuedTransaction {
            sender_address: tx.sender_address,
            nonce: tx.nonce,
            tx_hash: tx.tx_hash,
            tip: tx.tip,
        }
    }
}

// TODO: remove when is used.
#[allow(dead_code)]
// Invariant: Transactions have strictly increasing nonces, without gaps.
// Assumption: Transactions are provided in the correct order.
#[derive(Default)]
pub struct AddressPriorityQueue(VecDeque<ThinTransaction>);

// TODO: remove when is used.
#[allow(dead_code)]
impl AddressPriorityQueue {
    pub fn push(&mut self, tx: ThinTransaction) {
        if let Some(last_tx) = self.0.back() {
            assert_eq!(
                tx.nonce,
                last_tx.nonce.try_increment().expect("Nonce overflow."),
                "Nonces must be strictly increasing without gaps."
            );
        }

        self.0.push_back(tx);
    }

    pub fn top(&self) -> Option<&ThinTransaction> {
        self.0.front()
    }

    pub fn pop_front(&mut self) -> Option<ThinTransaction> {
        self.0.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, tx: &ThinTransaction) -> bool {
        self.0.contains(tx)
    }
}
