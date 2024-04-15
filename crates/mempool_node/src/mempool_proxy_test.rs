mod tests {

    use std::sync::Arc;

    use tokio::task::JoinSet;

    use crate::{
        mempool::{AddTransactionCallType, AddTransactionReturnType, Mempool, MempoolTrait},
        mempool_proxy::MempoolProxy,
    };

    #[tokio::test]
    async fn test_mempool_simple_add_transaction() {
        let mempool = Mempool::default();
        let tx: AddTransactionCallType = 1;
        let expected_result: AddTransactionReturnType = 1;
        assert_eq!(mempool.add_transaction(tx).await, expected_result);
    }

    #[tokio::test]
    async fn test_proxy_simple_add_transaction() {
        let proxy = MempoolProxy::default();
        let tx: AddTransactionCallType = 1;
        let expected_result: AddTransactionReturnType = 1;
        assert_eq!(proxy.add_transaction(tx).await, expected_result);
    }

    #[tokio::test]
    async fn test_mempool_concurrent_add_transaction() {
        let mempool = Arc::new(Mempool::default());

        let mut tasks: JoinSet<_> = (0..5)
            .map(|_| {
                let mempool = mempool.clone();
                async move {
                    let tx: AddTransactionCallType = 1;
                    mempool.add_transaction(tx).await
                }
            })
            .collect();

        let mut results: Vec<AddTransactionReturnType> = vec![];
        while let Some(result) = tasks.join_next().await {
            results.push(result.unwrap());
        }

        results.sort();

        let expected_results: Vec<AddTransactionReturnType> = (1..=5).collect();
        assert_eq!(results, expected_results);
    }

    #[tokio::test]
    async fn test_proxy_concurrent_add_transaction() {
        let proxy = MempoolProxy::default();

        let mut tasks: JoinSet<_> = (0..5)
            .map(|_| {
                let proxy = proxy.clone();
                async move {
                    let tx: AddTransactionCallType = 1;
                    proxy.add_transaction(tx).await
                }
            })
            .collect();

        let mut results: Vec<AddTransactionReturnType> = vec![];
        while let Some(result) = tasks.join_next().await {
            results.push(result.unwrap());
        }

        results.sort();

        let expected_results: Vec<AddTransactionReturnType> = (1..=5).collect();
        assert_eq!(results, expected_results);
    }
}
