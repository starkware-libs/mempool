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
    Account, AccountState, GatewayMessage, GatewayNetworkComponent, MempoolMessage,
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
    let (tx_gateway_2_mempool, rx_gateway_2_mempool) = channel::<GatewayMessage>(1);
    let (tx_mempool_2_gateway, rx_mempool_2_gateway) = channel::<MempoolMessage>(1);

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
        let gateway_2_mempool = GatewayMessage::AddTx(internal_tx, account);
        gateway_network.send(gateway_2_mempool).await.unwrap();
    })
    .await
    .unwrap();

    let mempool_message = MempoolMessage::from(
        task::spawn(async move { mempool_network.recv().await })
            .await
            .unwrap()
            .unwrap(),
    );

    match mempool_message {
        MempoolMessage::AddTx(tx, _) => match tx {
            InternalTransaction::Invoke(invoke_tx) => {
                assert_eq!(invoke_tx.tx_hash, tx_hash);
            }
            _ => panic!("Received a non-invoke transaction in AddTx"),
        },
        _ => panic!("Unhandled message type in mempool"),
    }
}

const TEST_FILES_FOLDER: &str = "./tests/fixtures";
// TODO(Ayelet): Replace the use of the JSON files with generated instances, then serialize these
// into JSON for testing.
#[rstest]
#[case::invoke(&Path::new(TEST_FILES_FOLDER).join("invoke_v3.json"), "INVOKE")]
#[tokio::test]
async fn test_end_to_end(#[case] json_file_path: &Path, #[case] expected_response: &str) {
    let (tx_gateway_2_mempool, rx_gateway_2_mempool) = channel::<GatewayMessage>(1);
    let (tx_mempool_2_gateway, rx_mempool_2_gateway) = channel::<MempoolMessage>(1);

    let network_gateway = GatewayNetworkComponent::new(tx_gateway_2_mempool, rx_mempool_2_gateway);
    let network_mempool = MempoolNetworkComponent::new(tx_mempool_2_gateway, rx_gateway_2_mempool);

    // Initialize Gateway.
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
        config: gateway_config.clone(),
        network: network_gateway,
        stateless_transaction_validator_config,
    };

    // Setup server
    tokio::spawn(async move { gateway.build_server().await });

    // Ensure the server has time to start up
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

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

    assert_eq!(response.status(), StatusCode::default());

    let res = response.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(res, expected_response.as_bytes());

    // Initialize Mempool.
    let mempool = Arc::new(Mutex::new(Mempool::new([], network_mempool)));

    let mempool_clone = mempool.clone();
    let listener_task = task::spawn(async move {
        mempool_clone.lock().await.start_network_listener().await;
    });

    // Assuming a way to stop the listener or simulate complete conditions
    let _ = tokio::time::timeout(Duration::from_secs(10), listener_task).await;

    // TODO: uncomment the below code.
    // assert!(
    //     result.is_err(),
    //     "Expected the listener to be still running, but it stopped"
    // );

    let txs = mempool.lock().await.get_txs(1).unwrap();
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].tip(), Some(Tip(0)));
}
