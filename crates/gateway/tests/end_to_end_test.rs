use starknet_api::{
    data_availability::DataAvailabilityMode,
    internal_transaction::{InternalInvokeTransaction, InternalTransaction},
    transaction::{
        InvokeTransaction, InvokeTransactionV3, ResourceBounds, ResourceBoundsMapping,
        TransactionHash,
    },
};
use tokio::{sync::mpsc::channel, task};

use mempool_infra::network_component::CommunicationInterface;

use starknet_mempool_types::mempool_types::{
    AccountState, GatewayMessage, GatewayNetworkComponent, MempoolMessage, MempoolNetworkComponent,
};

pub fn create_tx_for_testing() -> InternalTransaction {
    let tx = InvokeTransactionV3 {
        resource_bounds: ResourceBoundsMapping::try_from(vec![
            (
                starknet_api::transaction::Resource::L1Gas,
                ResourceBounds::default(),
            ),
            (
                starknet_api::transaction::Resource::L2Gas,
                ResourceBounds::default(),
            ),
        ])
        .expect("Resource bounds mapping has unexpected structure."),
        signature: Default::default(),
        nonce: Default::default(),
        sender_address: Default::default(),
        calldata: Default::default(),
        nonce_data_availability_mode: DataAvailabilityMode::L1,
        fee_data_availability_mode: DataAvailabilityMode::L1,
        paymaster_data: Default::default(),
        account_deployment_data: Default::default(),
        tip: Default::default(),
    };

    InternalTransaction::Invoke(InternalInvokeTransaction {
        tx: InvokeTransaction::V3(tx),
        tx_hash: Default::default(),
        only_query: false,
    })
}

#[tokio::test]
async fn test_send_and_receive() {
    let (tx_gateway_2_mempool, rx_gateway_2_mempool) = channel::<GatewayMessage>(1);
    let (tx_mempool_2_gateway, rx_mempool_2_gateway) = channel::<MempoolMessage>(1);

    let network_gateway = GatewayNetworkComponent::new(tx_gateway_2_mempool, rx_mempool_2_gateway);
    let mut network_mempool =
        MempoolNetworkComponent::new(tx_mempool_2_gateway, rx_gateway_2_mempool);

    let internal_tx = create_tx_for_testing();
    task::spawn(async move {
        let state: AccountState = AccountState {};
        let gateway_2_mempool = GatewayMessage::AddTx(internal_tx, state);
        network_gateway.send(gateway_2_mempool).await.unwrap();
    })
    .await
    .unwrap();

    let mempool_message = MempoolMessage::from(
        task::spawn(async move { network_mempool.recv().await })
            .await
            .unwrap()
            .unwrap(),
    );

    match mempool_message {
        MempoolMessage::AddTx(tx, state) => match tx {
            InternalTransaction::Invoke(invoke_tx) => {
                if invoke_tx.tx_hash != TransactionHash::default() {
                    panic!("Unexpected transaction hash: {:?}", invoke_tx.tx_hash);
                }
                println!("Transaction: {:?}, state: {:?}", invoke_tx, state);
            }
            _ => panic!("Received a non-invoke transaction in AddTx"),
        },
        _ => panic!("Unhandled message type in mempool"),
    }
}
