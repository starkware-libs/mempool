mod test {
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use crate::{
        mempool::{self, MemPool},
        proxy_mempool,
    };

    #[tokio::test]
    async fn test_proxy_add_transaction() {
        let mempool = Arc::new(Mutex::new(mempool::DummyActualMemPool::new()));
        let mut proxy = proxy_mempool::ProxyMemPool::new(mempool);

        assert!(proxy.add_transaction(1).await);
    }
}
