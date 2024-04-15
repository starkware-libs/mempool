mod tests {

    use tokio::task::JoinSet;

    use crate::{
        mempool::{AddTransactionCallType, AddTransactionReturnType, MempoolTrait},
        mempool_proxy::MempoolProxy,
    };

    #[tokio::test]
    async fn test_proxy_simple_add_transaction() {
        let mut proxy = MempoolProxy::default();
        let tx: AddTransactionCallType = 1;
        let expect_result: AddTransactionReturnType = 1;
        assert_eq!(proxy.add_transaction(tx).await, expect_result);
    }

    #[tokio::test]
    async fn test_proxy_concurrent_add_transaction() {
        let proxy = MempoolProxy::default();

        let mut tasks: JoinSet<_> = (0..5)
            .map(|_| {
                let mut proxy = proxy.clone();
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
