use blockifier::test_utils::contracts::FeatureContract;
use blockifier::test_utils::CairoVersion;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_integration_tests::integration_test_utils::setup_with_tx_generation;

#[tokio::test]
async fn test_end_to_end() {
    let accounts = [
        FeatureContract::AccountWithoutValidations(CairoVersion::Cairo1),
        FeatureContract::AccountWithoutValidations(CairoVersion::Cairo0),
    ];
    let (mock_running_system, mut tx_generator) = setup_with_tx_generation(&accounts).await;
    let mut expected_tx_hashes = Vec::new();

    let account0_deploy_nonce0 = &tx_generator.account_with_id(0).generate_default_deploy_account();
    expected_tx_hashes.push(
        mock_running_system
            .assert_add_tx_success(account0_deploy_nonce0).await
    );

    let account0_invoke_nonce1 = tx_generator.account_with_id(0).generate_default();
    mock_running_system
        .assert_add_tx_success(&account0_invoke_nonce1)
        .await

    // FIXME: nonce=0 should always be deploy-account, but this currently goes into the queue.
    // Figure out why this is currently allowed, fix it or remove this comment.
    let account1_invoke_nonce0 = tx_generator.account_with_id(1).generate_default();
    expected_tx_hashes.push(
        mock_running_system
            .assert_add_tx_success(&account1_invoke_nonce0)
            .await,
    );

    let account0_invoke_nonce2 = tx_generator.account_with_id(0).generate_default();
    mock_running_system
        .assert_add_tx_success(&account0_invoke_nonce2)
        .await;

    let mempool_txs = mock_running_system.get_txs(4).await;
    // Only the nonce-0 txs are returned, because we haven't merged queue-replenishment yet.
    // This assert should be replaced with 4 once queue-replenishment is merged, also add a tx hole
    // at that point, and ensure the assert doesn't change due to that.
    assert_eq!(mempool_txs.len(), 2);
    let mut actual_tx_hashes: Vec<TransactionHash> =
        mempool_txs.iter().map(|tx| tx.tx_hash).collect();
    actual_tx_hashes.sort();
    expected_tx_hashes.sort();
    assert_eq!(expected_tx_hashes, actual_tx_hashes);
}
