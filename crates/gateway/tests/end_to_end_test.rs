use mempool_infra::network_component::CommunicationInterface;
use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::{Tip, TransactionHash};
use starknet_mempool_types::mempool_types::{
    Account, AccountState, GatewayNetworkComponent, GatewayToMempoolMessage,
    MempoolNetworkComponent, MempoolToGatewayMessage,
};
use starknet_mempool_types::utils::create_thin_tx_for_testing;
use tokio::sync::mpsc::channel;
use tokio::task;

pub fn create_default_account() -> Account {
    Account { address: ContractAddress::default(), state: AccountState { nonce: Nonce::default() } }
}

#[tokio::test]
async fn test_send_and_receive() {
    let (tx_gateway_to_mempool, rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (tx_mempool_to_gateway, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);

    let gateway_network =
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway);
    let mut mempool_network =
        MempoolNetworkComponent::new(tx_mempool_to_gateway, rx_gateway_to_mempool);

    let tx_hash = TransactionHash::default();
    let thin_tx = create_thin_tx_for_testing(Tip::default(), tx_hash, ContractAddress::default());
    let account = create_default_account();
    task::spawn(async move {
        let gateway_to_mempool = GatewayToMempoolMessage::AddTransaction(thin_tx, account);
        gateway_network.send(gateway_to_mempool).await.unwrap();
    })
    .await
    .unwrap();

    let mempool_message =
        task::spawn(async move { mempool_network.recv().await }).await.unwrap().unwrap();

    match mempool_message {
        GatewayToMempoolMessage::AddTransaction(tx, _) => {
            assert_eq!(tx.tx_hash, tx_hash);
        }
    }
}
