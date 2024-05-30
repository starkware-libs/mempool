use std::collections::{BTreeMap, HashMap};

use starknet_api::core::{ContractAddress, Nonce};
use starknet_mempool_types::mempool_types::ThinTransaction;

#[derive(Clone, Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct TransactionStore(HashMap<ContractAddress, BTreeMap<Nonce, ThinTransaction>>);

impl TransactionStore {
    pub fn push(&mut self, tx: ThinTransaction) {
        self.entry(tx.sender_address).or_default().insert(tx.nonce, tx.clone());
    }

    pub fn remove(
        &mut self,
        sender_address: &ContractAddress,
        nonce: &Nonce,
    ) -> Option<ThinTransaction> {
        if let Some(tree) = self.0.get_mut(sender_address) {
            return tree.remove(nonce);
        }
        None
    }

    pub fn get(
        &mut self,
        sender_address: &ContractAddress,
        nonce: &Nonce,
    ) -> Option<&ThinTransaction> {
        if let Some(tree) = self.0.get(sender_address) {
            return tree.get(nonce);
        }
        None
    }
}
