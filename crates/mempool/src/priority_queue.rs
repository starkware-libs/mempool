#[cfg(test)]
#[path = "priority_queue_test.rs"]
pub mod priority_queue_test;

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use starknet_api::{
    internal_transaction::InternalTransaction,
    transaction::{
        DeclareTransaction, DeployAccountTransaction, InvokeTransaction, Tip, TransactionHash,
    },
};

fn get_tip(tx: &InternalTransaction) -> Tip {
    match tx {
        InternalTransaction::Declare(declare_tx) => match &declare_tx.tx {
            DeclareTransaction::V3(declare_tx_v3) => declare_tx_v3.tip,
            _ => panic!("Unexpected transaction version."),
        },
        InternalTransaction::DeployAccount(deploy_account_tx) => match &deploy_account_tx.tx {
            DeployAccountTransaction::V3(tx_v3) => tx_v3.tip,
            _ => panic!("Unexpected transaction version."),
        },
        InternalTransaction::Invoke(invoke_tx) => match &invoke_tx.tx {
            InvokeTransaction::V3(tx_v3) => tx_v3.tip,
            _ => panic!("Unexpected transaction version."),
        },
    }
}

fn get_tx_hash(tx: &InternalTransaction) -> TransactionHash {
    match tx {
        InternalTransaction::Declare(declare_tx) => declare_tx.tx_hash,
        InternalTransaction::DeployAccount(deploy_account_tx) => deploy_account_tx.tx_hash,
        InternalTransaction::Invoke(invoke_tx) => invoke_tx.tx_hash,
    }
}

#[derive(Clone, Debug)]
pub struct MempoolTransaction {
    pub tx: InternalTransaction,
    pub tx_hash: TransactionHash,
}

impl PartialEq for MempoolTransaction {
    fn eq(&self, other: &MempoolTransaction) -> bool {
        get_tip(&self.tx) == get_tip(&other.tx)
    }
}

impl Eq for MempoolTransaction {}

impl PartialOrd for MempoolTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MempoolTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        get_tip(&self.tx).cmp(&get_tip(&other.tx))
    }
}

#[derive(Clone, Default)]
pub struct PriorityQueue {
    heap: BinaryHeap<MempoolTransaction>,
    pub tx_hash_to_tx_map: HashMap<TransactionHash, InternalTransaction>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        PriorityQueue {
            heap: BinaryHeap::new(),
            tx_hash_to_tx_map: HashMap::new(),
        }
    }

    pub fn push(&mut self, tx: InternalTransaction) {
        let tx_hash = get_tx_hash(&tx);
        self.tx_hash_to_tx_map.insert(tx_hash, tx.clone());
        let mempool_tx = MempoolTransaction { tx, tx_hash };

        self.heap.push(mempool_tx);
    }

    // Removes and returns the transaction with the highest tip.
    pub fn pop(&mut self) -> Option<TransactionHash> {
        let mempool_tx = self.heap.pop();
        match mempool_tx {
            Some(mempool_tx) => {
                let tx_hash = mempool_tx.tx_hash;
                self.tx_hash_to_tx_map.remove(&tx_hash);
                Some(tx_hash)
            }
            None => None,
        }
    }
}
