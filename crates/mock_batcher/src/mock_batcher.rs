use starknet_mempool_types::communication::{MempoolClient, MempoolClientImpl};
use starknet_mempool_types::mempool_types::Account;


struct MockBatcher {
    mempool_client: MempoolClientImpl,
}

impl MockBatcher {
    pub fn run(&self) {
        loop of channel of integration setup test
        wait for it to call fetch_txs_from_mempool
    }

    fn get_txs(&self, channel) {
        // get
        self.mempool_client.get_txs();
    }
}
