use assert_matches::assert_matches;

use crate::errors::GatewayError;
use crate::gateway::Gateway;
use crate::gateway::GatewayConfig;
use crate::gateway::HandleContext;
use crate::test_utils::create_a_declare_tx;
use crate::transaction::ExternalTransaction;
use hyper::StatusCode;
use hyper::{Body, Request};
use starknet_api::core::ChainId;
use starknet_api::transaction::DeclareTransaction;
use starknet_api::transaction::Transaction;
use tokio::time::{delay_for, Duration};

#[tokio::test]
async fn test_invalid_request() {
    // Create a sample GET request for an invalid path
    let request = Request::get("/some_invalid_path")
        .body(Body::empty())
        .unwrap();
    let gateway = Gateway::new(GatewayConfig {
        bind_address: "0.0.0.0:8080".to_string(),
        chain_id: ChainId("SN_MAIN".to_owned()),
    });
    let response = gateway
        .handle_request(HandleContext {}, request)
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
    assert_eq!(
        String::from_utf8_lossy(&hyper::body::to_bytes(response.into_body()).await.unwrap()),
        "Not found."
    );
}

#[tokio::test]
async fn test_build_server() {
    let gateway = Gateway {
        gateway_config: GatewayConfig {
            bind_address: "0.0.0.0:8080".to_string(),
            chain_id: ChainId("SN_MAIN".to_owned()),
        },
    };

    tokio::spawn(async move {
        gateway.build_server().await.unwrap();
    });
    delay_for(Duration::from_secs(1)).await;

    let client = hyper::Client::new();
    let uri = "http://127.0.0.1:8080/is_alive".parse().unwrap();
    let response = client.get(uri).await.unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        String::from_utf8_lossy(&hyper::body::to_bytes(response.into_body()).await.unwrap()),
        "Server is alive"
    );
}

#[tokio::test]
async fn test_add_transaction_declare() {
    // Happy flow.
    // Create a POST request to /add_transaction with a valid JSON.
    let declare_tx = create_a_declare_tx();
    let external_tx =
        ExternalTransaction::new(Transaction::Declare(DeclareTransaction::V0(declare_tx)));

    let serialized = serde_json::to_string(&external_tx).unwrap();

    let valid_request = Request::post("/add_transaction")
        .body(Body::from(serialized))
        .unwrap();
    let gateway = Gateway::new(GatewayConfig {
        bind_address: "0.0.0.0:8080".to_string(),
        chain_id: ChainId("SN_MAIN".to_owned()),
    });
    let valid_response = gateway
        .handle_request(HandleContext {}, valid_request)
        .await
        .unwrap();

    assert_eq!(valid_response.status(), StatusCode::OK);
    let body_bytes = hyper::body::to_bytes(valid_response.into_body())
        .await
        .expect("Failed to read response body");
    assert_eq!(String::from_utf8_lossy(&body_bytes), "Declare");

    // Negative flow.
    // Create a POST request to /add_transaction with an invalid JSON.
    let invalid_serialized = "{ \"invalid\": \"data\" }".to_string();

    let invalid_request = Request::post("/add_transaction")
        .body(Body::from(invalid_serialized))
        .unwrap();

    let invalid_response = gateway
        .handle_request(HandleContext {}, invalid_request)
        .await;

    assert_matches!(
        invalid_response,
        Err(GatewayError::InvalidTransactionFormat)
    );
}

#[tokio::test]
async fn test_convert_external_transaction() {
    // Happy flow.
    // Create a POST request to /add_transaction with a valid JSON.
    let declare_tx = create_a_declare_tx();
    let tx = ExternalTransaction::new(Transaction::Declare(DeclareTransaction::V0(declare_tx)));

    let chain_id = ChainId("SN_MAIN".to_owned());
    let gateway = Gateway::new(GatewayConfig {
        bind_address: "0.0.0.0:8080".to_string(),
        chain_id: chain_id.clone(),
    });
    let internal_tx1 = gateway.convert_to_internal_tx(tx.clone(), chain_id.clone());
    let internal_tx2 = gateway.convert_to_internal_tx(tx.clone(), chain_id);

    assert!(internal_tx1.get_transaction_hash() == internal_tx2.get_transaction_hash());
    let chain_id = ChainId("SN_INTEGRATION".to_owned());
    let internal_tx3 = gateway.convert_to_internal_tx(tx, chain_id);
    assert!(internal_tx1.get_transaction_hash() != internal_tx3.get_transaction_hash());
}
