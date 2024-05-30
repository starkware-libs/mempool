use std::collections::{BTreeMap, HashMap};

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::mempool_types::ThinTransaction;

#[derive(Clone, Debug, Default)]
pub struct TransactionStore {
    store: HashMap<ContractAddress, BTreeMap<Nonce, ThinTransaction>>,
    tx_hash_2_tx: HashMap<TransactionHash, (ContractAddress, Nonce)>,
}

impl TransactionStore {
    pub fn push(&mut self, tx: ThinTransaction) {
        self.store.entry(tx.sender_address).or_default().insert(tx.nonce, tx.clone());
        self.tx_hash_2_tx.insert(tx.tx_hash, (tx.sender_address, tx.nonce));
    }

    pub fn remove(&mut self, tx_hash: &TransactionHash) -> Option<ThinTransaction> {
        let (address, nonce) = self.tx_hash_2_tx.remove(tx_hash).unwrap();
        if let Some(tree_map) = self.store.get_mut(&address) {
            return tree_map.remove(&nonce);
        }
        None
    }

    pub fn get(&mut self, tx_hash: &TransactionHash) -> Option<&ThinTransaction> {
        let (address, nonce) = self.tx_hash_2_tx.get(tx_hash).unwrap();
        if let Some(tree_map) = self.store.get(address) {
            return tree_map.get(nonce);
        }
        None
    }
}
