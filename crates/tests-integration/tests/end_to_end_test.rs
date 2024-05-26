use mempool_infra::network_component::CommunicationInterface;
use pretty_assertions::assert_eq;
use rstest::rstest;
use starknet_api::transaction::{Tip, TransactionHash};
use starknet_gateway::starknet_api_test_utils::invoke_tx;
use starknet_mempool_integration_tests::integration_test_setup::IntegrationTestSetup;
use starknet_mempool_types::mempool_types::{
    GatewayNetworkComponent, GatewayToMempoolMessage, MempoolInput, MempoolNetworkComponent,
    MempoolToGatewayMessage,
};
use tokio::sync::mpsc::channel;
use tokio::task;

#[tokio::test]
async fn test_send_and_receive() {
    let (tx_gateway_to_mempool, rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (tx_mempool_to_gateway, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);

    let gateway_network =
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway);
    let mut mempool_network =
        MempoolNetworkComponent::new(tx_mempool_to_gateway, rx_gateway_to_mempool);

    let tx_hash = TransactionHash::default();
    let mempool_input = MempoolInput::default();
    task::spawn(async move {
        let gateway_to_mempool = GatewayToMempoolMessage::AddTransaction(mempool_input);
        gateway_network.send(gateway_to_mempool).await.unwrap();
    })
    .await
    .unwrap();

    let mempool_message =
        task::spawn(async move { mempool_network.recv().await }).await.unwrap().unwrap();

    match mempool_message {
        GatewayToMempoolMessage::AddTransaction(mempool_input) => {
            assert_eq!(mempool_input.tx.tx_hash, tx_hash);
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_end_to_end() {
    let mock = IntegrationTestSetup::new().await;

    mock.assert_add_tx_success(&invoke_tx(), "INVOKE").await;

    // TODO: we need a better way of asserting external txs with their internal counterparts,
    // without having to compute hashes (maybe just assert the rest of the fields?).
    let mempool_txs = mock.get_txs(2).await;
    assert_eq!(mempool_txs.len(), 1);
    assert_eq!(mempool_txs[0].tip, Tip(0));
}
