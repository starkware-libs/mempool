use std::collections::HashMap;

use starknet_api::core::ContractAddress;
use starknet_mempool_types::mempool_types::ThinTransaction;

use crate::priority_queue::AddressPriorityQueue;

#[derive(Default)]
pub struct TransactionStore {
    pub address_to_queue: HashMap<ContractAddress, AddressPriorityQueue>,
}

impl TransactionStore {
    pub fn push(&mut self, tx: ThinTransaction) {
        let address_queue = self
            .address_to_queue
            .entry(tx.sender_address)
            .or_insert_with(|| AddressPriorityQueue(Vec::new()));
        address_queue.push(tx);
    }

    pub fn pop(&mut self, address: &ContractAddress) -> Option<ThinTransaction> {
        if let Some(queue) = self.address_to_queue.get_mut(address) {
            let popped_tx = queue.pop();
            if queue.is_empty() {
                self.address_to_queue.remove(address);
            }
            popped_tx
        } else {
            None
        }
    }
}
