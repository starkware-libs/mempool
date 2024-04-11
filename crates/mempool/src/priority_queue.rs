use std::{cmp::Ordering, collections::BTreeSet};

use starknet_api::{
    internal_transaction::InternalTransaction,
    transaction::{DeclareTransaction, DeployAccountTransaction, InvokeTransaction, Tip},
};

// Assumption: for the MVP only one transaction from the same contract class can be in the mempool
// at a time. When this changes, saving the transactions themselves on the queu might no longer be
// appropriate, because we'll also need to stores transactions without indexing them. For example,
// transactions with future nonces will need to be stored, and potentially indexed on block commits.
#[derive(Clone, Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct PriorityQueue(BTreeSet<Transaction>);

impl PriorityQueue {
    pub fn new() -> Self {
        PriorityQueue(BTreeSet::new())
    }

    pub fn push(&mut self, tx: InternalTransaction) {
        let mempool_tx = Transaction(tx);
        self.insert(mempool_tx);
    }

    pub fn pop(&mut self, n_txs: usize) -> Vec<InternalTransaction> {
        let mut txs = Vec::new();
        for _ in 0..n_txs {
            match self.pop_last().map(|tx| tx.0) {
                Some(tx) => txs.push(tx),
                None => break,
            }
        }
        txs
    }
}

#[derive(Clone, Debug, derive_more::Deref)]
pub struct Transaction(pub InternalTransaction);

impl Transaction {
    fn tip(&self) -> Tip {
        match &self.0 {
            InternalTransaction::Declare(declare_tx) => match &declare_tx.tx {
                DeclareTransaction::V3(tx_v3) => tx_v3.tip,
                _ => unimplemented!(),
            },
            InternalTransaction::DeployAccount(deploy_account_tx) => match &deploy_account_tx.tx {
                DeployAccountTransaction::V3(tx_v3) => tx_v3.tip,
                _ => unimplemented!(),
            },
            InternalTransaction::Invoke(invoke_tx) => match &invoke_tx.tx {
                InvokeTransaction::V3(tx_v3) => tx_v3.tip,
                _ => unimplemented!(),
            },
        }
    }
}

// Compare transactions based on their tip only, which implies `Eq`, because `tip` is uint.
impl PartialEq for Transaction {
    fn eq(&self, other: &Transaction) -> bool {
        self.tip() == other.tip()
    }
}

/// Marks Transaction as capable of strict equality comparisons, signaling to the compiler it
/// adheres to equality semantics.
// Note: this depends on the implementation of `PartialEq`, see its docstring.
impl Eq for Transaction {}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tip().cmp(&other.tip())
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
