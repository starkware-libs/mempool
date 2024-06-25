use blockifier::test_utils::CairoVersion;
use starknet_gateway::starknet_api_test_utils::invoke_tx;
use starknet_mempool_integration_tests::integration_test_setup::IntegrationTestSetup;

#[tokio::test]
async fn test_end_to_end() {
    let mut mock_running_system = IntegrationTestSetup::new().await;

    let mut expected_tx_hash = Vec::new();
    expected_tx_hash
        .push(mock_running_system.assert_add_tx_success(&invoke_tx(CairoVersion::Cairo0)).await);
    expected_tx_hash
        .push(mock_running_system.assert_add_tx_success(&invoke_tx(CairoVersion::Cairo1)).await);
 
    let mempool_txs = mock_running_system.get_txs(4).await;
    assert_eq!(mempool_txs.len(), 2);
    expected_tx_hash.iter().for_each(|tx_hash| {
        assert!(mempool_txs.iter().any(|tx| tx.tx_hash == *tx_hash), "Did not get tx_hash: {:?} from mempool", tx_hash);
    });
}
