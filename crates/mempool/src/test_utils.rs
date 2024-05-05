use crate::mempool::Mempool;
use starknet_api::internal_transaction::InternalTransaction;

pub struct MockBatcher {
    _mempool: Mempool,
}

impl MockBatcher {
    pub fn _new(mempool: Mempool) -> Self {
        MockBatcher { _mempool: mempool }
    }

    fn _retrieve_txs(&self, txs: Vec<InternalTransaction>) {
        println!("Sending transactions: {:?}", txs);
    }
}
