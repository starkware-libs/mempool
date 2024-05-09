use starknet_api::{
    core::{ContractAddress, Nonce},
    internal_transaction::InternalTransaction,
};
use starknet_gateway::starknet_api_test_utils::create_internal_tx_for_testing;
use tokio::{sync::mpsc::channel, task};

use mempool_infra::network_component::CommunicationInterface;

use starknet_mempool_types::mempool_types::{
    Account, AccountState, Gateway2MempoolMessage, GatewayNetworkComponent, Mempool2GatewayMessage,
    MempoolNetworkComponent,
};

pub fn create_default_account() -> Account {
    Account {
        address: ContractAddress::default(),
        state: AccountState {
            nonce: Nonce::default(),
        },
    }
}

#[tokio::test]
async fn test_send_and_receive() {
    let (tx_gateway_2_mempool, rx_gateway_2_mempool) = channel::<Gateway2MempoolMessage>(1);
    let (tx_mempool_2_gateway, rx_mempool_2_gateway) = channel::<Mempool2GatewayMessage>(1);

    let gateway_network = GatewayNetworkComponent::new(tx_gateway_2_mempool, rx_mempool_2_gateway);
    let mut mempool_network =
        MempoolNetworkComponent::new(tx_mempool_2_gateway, rx_gateway_2_mempool);

    let internal_tx = create_internal_tx_for_testing();
    let tx_hash = match internal_tx {
        InternalTransaction::Invoke(ref invoke_transaction) => Some(invoke_transaction.tx_hash),
        _ => None,
    }
    .unwrap();
    let account = create_default_account();
    task::spawn(async move {
        let gateway_2_mempool = Gateway2MempoolMessage::AddTx(internal_tx, account);
        gateway_network.send(gateway_2_mempool).await.unwrap();
    })
    .await
    .unwrap();

    let mempool_message = task::spawn(async move { mempool_network.recv().await })
        .await
        .unwrap()
        .unwrap();

    match mempool_message {
        Gateway2MempoolMessage::AddTx(tx, _) => match tx {
            InternalTransaction::Invoke(invoke_tx) => {
                assert_eq!(invoke_tx.tx_hash, tx_hash);
            }
            _ => panic!("Received a non-invoke transaction in AddTx"),
        },
    }
}
