use std::collections::{BTreeMap, HashMap};

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::ThinTransaction;

#[derive(Clone, Debug, Default)]
pub struct TransactionStore {
    store: HashMap<ContractAddress, BTreeMap<Nonce, ThinTransaction>>,
    tx_hash_2_tx: HashMap<TransactionHash, (ContractAddress, Nonce)>,
}

impl TransactionStore {
    pub fn push(&mut self, tx: ThinTransaction) -> Result<(), MempoolError> {
        let account_store = self.store.entry(tx.sender_address).or_default();
        // TODO(Mohammad): Allow overriding a previous transaction.
        if account_store.contains_key(&tx.nonce) {
            return Err(MempoolError::DuplicateTransaction { tx_hash: tx.tx_hash });
        }
        account_store.insert(tx.nonce, tx.clone());
        self.tx_hash_2_tx.insert(tx.tx_hash, (tx.sender_address, tx.nonce));
        Ok(())
    }

    pub fn remove(&mut self, tx_hash: &TransactionHash) -> Result<ThinTransaction, MempoolError> {
        if let Some((address, nonce)) = self.tx_hash_2_tx.remove(tx_hash) {
            if let Some(tree_map) = self.store.get_mut(&address) {
                if let Some(tx) = tree_map.remove(&nonce) {
                    return Ok(tx);
                }
            }
        }
        Err(MempoolError::TransactionNotFound { tx_hash: *tx_hash })
    }

    pub fn get(&mut self, tx_hash: &TransactionHash) -> Result<&ThinTransaction, MempoolError> {
        let (address, nonce) = self.tx_hash_2_tx.get(tx_hash).unwrap();
        if let Some(tree_map) = self.store.get(address) {
            if let Some(tx) = tree_map.get(nonce) {
                return Ok(tx);
            }
        }
        Err(MempoolError::TransactionNotFound { tx_hash: *tx_hash })
    }
}
