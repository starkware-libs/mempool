use std::marker::{Send, Sync};
use std::sync::Arc;

use crate::proxy_component::{
    AddTransactionCallType, AddTransactionReturnType, AddTransactionTrait, CompA, CompAProxy,
    GetTransactionReturnType, GetTransactionTrait,
};

use tokio::task::JoinSet;

async fn test_mempool_single_thread_add_transaction<T>(comp_a: T)
where
    T: AddTransactionTrait + GetTransactionTrait,
{
    let tx: AddTransactionCallType = 37;
    let expected_add_result: AddTransactionReturnType = 1;
    assert_eq!(comp_a.add_transaction(tx).await, expected_add_result);

    let expected_get_result: GetTransactionReturnType = GetTransactionReturnType { value: 1 };
    assert_eq!(comp_a.get_transaction().await, expected_get_result);
}

async fn test_mempool_concurrent_add_transaction<T>(comp_a: Arc<T>)
where
    T: AddTransactionTrait + GetTransactionTrait + Send + Sync + 'static,
{
    let mut tasks: JoinSet<_> = (0..5)
        .map(|_| {
            let comp_a = comp_a.clone();
            async move {
                let tx: AddTransactionCallType = 1;
                comp_a.add_transaction(tx).await
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

    let expected_get_result: GetTransactionReturnType = GetTransactionReturnType { value: 5 };
    assert_eq!(comp_a.get_transaction().await, expected_get_result);
}

#[tokio::test]
async fn test_direct_mempool_single_thread_add_transaction() {
    test_mempool_single_thread_add_transaction(CompA::default()).await;
}

#[tokio::test]
async fn test_proxy_mempool_single_thread_add_transaction() {
    test_mempool_single_thread_add_transaction(CompAProxy::default()).await;
}

#[tokio::test]
async fn test_direct_mempool_concurrent_add_transaction() {
    test_mempool_concurrent_add_transaction(Arc::new(CompA::default())).await;
}

#[tokio::test]
async fn test_proxy_mempool_concurrent_add_transaction() {
    test_mempool_concurrent_add_transaction(Arc::new(CompAProxy::default())).await;
}
