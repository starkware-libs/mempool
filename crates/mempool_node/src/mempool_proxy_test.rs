mod tests {
    use std::time::Duration;
    use tokio::task;
    use tokio::time::sleep;

    use crate::{mempool::MempoolTrait, mempool_proxy::MempoolProxy};

    #[tokio::test]
    async fn test_proxy_simple_add_transaction() {
        let mut proxy = MempoolProxy::default();
        assert_eq!(proxy.add_transaction(1).await, 1);
    }

    #[tokio::test]
    async fn test_concurrent_add_transaction() {
        let mut proxy1 = MempoolProxy::default();
        let mut proxy2 = proxy1.clone();

        task::spawn(async move {
            proxy2.add_transaction(2).await;
        });

        assert_eq!(proxy1.add_transaction(1).await, 1);
        sleep(Duration::from_millis(1)).await;
        assert_eq!(proxy1.add_transaction(3).await, 3);
    }
}
