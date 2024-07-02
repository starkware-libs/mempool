use blockifier::test_utils::CairoVersion;
use starknet_mempool_integration_tests::integration_test_setup::IntegrationTestSetup;
use test_utils::starknet_api_test_utils::{deploy_account_tx, invoke_tx};

#[tokio::test]
async fn test_end_to_end() {
    let mock_running_system = IntegrationTestSetup::new(1).await;

    let mut expected_tx_hashes = Vec::new();
    expected_tx_hashes
        .push(mock_running_system.assert_add_tx_success(&invoke_tx(CairoVersion::Cairo0)).await);
    expected_tx_hashes
        .push(mock_running_system.assert_add_tx_success(&invoke_tx(CairoVersion::Cairo1)).await);
    expected_tx_hashes.push(mock_running_system.assert_add_tx_success(&deploy_account_tx()).await);

    mock_running_system.trigger_batcher().await;
    // TODO: Get txs from batcher and assert that they are the expected ones.
}
