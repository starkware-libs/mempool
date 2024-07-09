use blockifier::test_utils::CairoVersion;
use mempool_test_utils::starknet_api_test_utils::{deploy_account_tx, invoke_tx};
use starknet_api::transaction::TransactionHash;
use starknet_mempool_integration_tests::integration_test_setup::IntegrationTestSetup;

#[tokio::test]
async fn test_end_to_end() {
    let n_accounts = 1;
    let mock_running_system = IntegrationTestSetup::new(n_accounts).await;

    let mut expected_tx_hashes = Vec::new();
    expected_tx_hashes
        .push(mock_running_system.assert_add_tx_success(&invoke_tx(CairoVersion::Cairo0)).await);
    expected_tx_hashes
        .push(mock_running_system.assert_add_tx_success(&invoke_tx(CairoVersion::Cairo1)).await);
    expected_tx_hashes.push(mock_running_system.assert_add_tx_success(&deploy_account_tx()).await);

    let mempool_txs = mock_running_system.get_txs(4).await;
    assert_eq!(mempool_txs.len(), 3);
    let mut actual_tx_hashes: Vec<TransactionHash> =
        mempool_txs.iter().map(|tx| tx.tx_hash).collect();
    actual_tx_hashes.sort();
    expected_tx_hashes.sort();
    assert_eq!(expected_tx_hashes, actual_tx_hashes);
}

// Make sure we have the arbitrary precision feature of serde_json.
#[test]
fn serialization_precision() {
    let input =
        "{\"value\":244116128358498188146337218061232635775543270890529169229936851982759783745}";
    let serialized = serde_json::from_str::<serde_json::Value>(input).unwrap();
    let deserialized = serde_json::to_string(&serialized).unwrap();
    assert_eq!(input, deserialized);
}
