use axum::body::{Body, HttpBody};
use axum::http::{Request, StatusCode};
use hyper::Client;
use starknet_api::transaction::Tip;
use starknet_gateway::stateless_transaction_validator::StatelessTransactionValidatorConfig;
use starknet_gateway::{config::GatewayConfig, gateway::Gateway};
use starknet_mempool::mempool::Mempool;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::time::sleep;

use hyper::Response;
use rstest::rstest;

use std::fs;
use std::path::Path;

use starknet_api::{
    core::{ContractAddress, Nonce},
    data_availability::DataAvailabilityMode,
    internal_transaction::{InternalInvokeTransaction, InternalTransaction},
    transaction::{InvokeTransaction, InvokeTransactionV3, ResourceBounds, ResourceBoundsMapping},
};
use tokio::{sync::mpsc::channel, task};

use mempool_infra::network_component::CommunicationInterface;

use starknet_mempool_types::mempool_types::{
    Account, AccountState, GatewayNetworkComponent, GatewayToMempoolMessage,
    MempoolNetworkComponent, MempoolToGatewayMessage,
};

pub fn create_default_account() -> Account {
    Account {
        address: ContractAddress::default(),
        state: AccountState {
            nonce: Nonce::default(),
        },
    }
}

pub fn create_internal_tx_for_testing() -> InternalTransaction {
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
    let (tx_gateway_to_mempool, rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (tx_mempool_to_gateway, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);

    let gateway_network =
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway);
    let mut mempool_network =
        MempoolNetworkComponent::new(tx_mempool_to_gateway, rx_gateway_to_mempool);

    let internal_tx = create_internal_tx_for_testing();
    let tx_hash = match internal_tx {
        InternalTransaction::Invoke(ref invoke_transaction) => Some(invoke_transaction.tx_hash),
        _ => None,
    }
    .unwrap();
    let account = create_default_account();
    task::spawn(async move {
        let gateway_to_mempool = GatewayToMempoolMessage::AddTx(internal_tx, account);
        gateway_network.send(gateway_to_mempool).await.unwrap();
    })
    .await
    .unwrap();

    let mempool_message = task::spawn(async move { mempool_network.recv().await })
        .await
        .unwrap()
        .unwrap();

    match mempool_message {
        GatewayToMempoolMessage::AddTx(tx, _) => match tx {
            InternalTransaction::Invoke(invoke_tx) => {
                assert_eq!(invoke_tx.tx_hash, tx_hash);
            }
            _ => panic!("Received a non-invoke transaction in AddTx"),
        },
    }
}

const TEST_FILES_FOLDER: &str = "./tests/fixtures";

fn initialize_network_channels() -> (GatewayNetworkComponent, MempoolNetworkComponent) {
    let (tx_gateway_to_mempool, rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (tx_mempool_to_gateway, rx_mempool_to_gateway) = channel::<MempoolToGatewayMessage>(1);

    (
        GatewayNetworkComponent::new(tx_gateway_to_mempool, rx_mempool_to_gateway),
        MempoolNetworkComponent::new(tx_mempool_to_gateway, rx_gateway_to_mempool),
    )
}

async fn set_up_gateway(network: GatewayNetworkComponent) -> (IpAddr, u16) {
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = 3000;
    let gateway_config: GatewayConfig = GatewayConfig { ip, port };
    let stateless_transaction_validator_config = StatelessTransactionValidatorConfig {
        validate_non_zero_l1_gas_fee: true,
        max_calldata_length: 10,
        max_signature_length: 2,
        ..Default::default()
    };

    let gateway = Gateway {
        config: gateway_config,
        network,
        stateless_transaction_validator_config,
    };

    // Setup server
    tokio::spawn(async move { gateway.build_server().await });

    // Ensure the server has time to start up
    sleep(Duration::from_millis(1000)).await;
    (ip, port)
}

async fn send_and_verify_transaction(
    ip: IpAddr,
    port: u16,
    json_file_path: &Path,
    expected_response: &str,
) {
    let tx_json = fs::read_to_string(json_file_path).unwrap();
    let request = Request::builder()
        .method("POST")
        .uri(format!("http://{}", SocketAddr::from((ip, port))) + "/add_transaction")
        .header("content-type", "application/json")
        .body(Body::from(tx_json))
        .unwrap();

    // Create a client
    let client = Client::new();

    // Send a POST request with the transaction data as the body
    let response: Response<Body> = client.request(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let res = response.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(res, expected_response.as_bytes());
}

#[rstest]
#[tokio::test]
async fn test_end_to_end() {
    let (gateway_to_mempool_network, mempool_to_gateway_network) = initialize_network_channels();

    // Initialize Gateway.
    let (ip, port) = set_up_gateway(gateway_to_mempool_network).await;

    // Send a transaction.
    let invoke_json = &Path::new(TEST_FILES_FOLDER).join("invoke_v3.json");
    send_and_verify_transaction(ip, port, invoke_json, "INVOKE").await;

    // Initialize Mempool.
    let mempool = Arc::new(Mutex::new(Mempool::new([], mempool_to_gateway_network)));

    let mempool_clone = mempool.clone();
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();
    let _listener_task = task::spawn(async move {
        let mut lock = mempool_clone.lock().await;
        lock.start_network_listener(notify_clone).await;
    });

    // Wait for the listener to receive the transactions.
    sleep(Duration::from_secs(2)).await;
    notify.notify_one();

    let txs = mempool.lock().await.get_txs(1).unwrap();
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].tip(), Some(Tip(0)));
}
