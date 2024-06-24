use std::cmp::Ordering;
#[cfg(any(feature = "testing", test))]
use std::collections::btree_set::Iter;
use std::collections::{BTreeSet, HashMap, VecDeque};

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::{Tip, TransactionHash};
use starknet_mempool_types::mempool_types::ThinTransaction;
// Assumption: for the MVP only one transaction from the same contract class can be in the mempool
// at a time. When this changes, saving the transactions themselves on the queu might no longer be
// appropriate, because we'll also need to stores transactions without indexing them. For example,
// transactions with future nonces will need to be stored, and potentially indexed on block commits.

#[derive(Clone, Debug, Default)]
pub struct TransactionPriorityQueue {
    // Priority queue of transactions with associated priority.
    queue: BTreeSet<PrioritizedTransaction>,
    // FIX: Set of account addresses for efficient existence checks.
    address_to_nonce: HashMap<ContractAddress, Nonce>,
}

impl TransactionPriorityQueue {
    /// Adds a transaction to the mempool, ensuring unique keys.
    /// Panics: if given a duplicate tx.
    pub fn push(&mut self, tx: PrioritizedTransaction) {
        self.address_to_nonce.insert(tx.address, tx.nonce);
        assert!(self.queue.insert(tx), "Keys should be unique; duplicates are checked prior.");
    }

    // TODO(gilad): remove collect
    pub fn pop_last_chunk(&mut self, n_txs: usize) -> Vec<PrioritizedTransaction> {
        let txs: Vec<PrioritizedTransaction> =
            (0..n_txs).filter_map(|_| self.queue.pop_last()).collect();

        for tx in txs.iter() {
            self.address_to_nonce.remove(&tx.address);
        }

        txs
    }

    #[cfg(any(feature = "testing", test))]
    pub fn iter(&self) -> Iter<'_, PrioritizedTransaction> {
        self.queue.iter()
    }

    // TODO(Mohammad): delete once the mempool is used. It will be used in Mempool's
    // `get_next_eligible_tx`.
    #[allow(dead_code)]
    pub fn get_nonce(&self, address: &ContractAddress) -> Option<&Nonce> {
        self.address_to_nonce.get(address)
    }
}

#[derive(Clone, Debug, Default)]
pub struct PrioritizedTransaction {
    pub address: ContractAddress,
    pub nonce: Nonce,
    pub tx_hash: TransactionHash,
    pub tip: Tip,
}

/// Compare transactions based only on their tip, a uint, using the Eq trait. It ensures that two
/// tips are either exactly equal or not.
impl PartialEq for PrioritizedTransaction {
    fn eq(&self, other: &PrioritizedTransaction) -> bool {
        self.tip == other.tip && self.tx_hash == other.tx_hash
    }
}

/// Marks this struct as capable of strict equality comparisons, signaling to the compiler it
/// adheres to equality semantics.
// Note: this depends on the implementation of `PartialEq`, see its docstring.
impl Eq for PrioritizedTransaction {}

impl Ord for PrioritizedTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tip.cmp(&other.tip).then_with(|| self.tx_hash.cmp(&other.tx_hash))
    }
}

impl PartialOrd for PrioritizedTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<ThinTransaction> for PrioritizedTransaction {
    fn from(tx: ThinTransaction) -> Self {
        PrioritizedTransaction {
            address: tx.sender_address,
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
