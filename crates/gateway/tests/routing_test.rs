use axum::body::{Body, Bytes, HttpBody};
use axum::http::{Request, StatusCode};
use hyper::Client;
use starknet_mempool::mempool::Mempool;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use hyper::Response;
use pretty_assertions::assert_str_eq;
use rstest::rstest;
use starknet_gateway::gateway::app;
use starknet_gateway::gateway::{Gateway, GatewayConfig};

use std::fs;
use std::path::Path;
use tower::ServiceExt;

const TEST_FILES_FOLDER: &str = "./tests/fixtures";

// TODO(Ayelet): Replace the use of the JSON files with generated instances, then serialize these
// into JSON for testing.
#[rstest]
#[case::declare(&Path::new(TEST_FILES_FOLDER).join("declare_v3.json"), "DECLARE")]
#[case::deploy_account(
    &Path::new(TEST_FILES_FOLDER).join("deploy_account_v3.json"),
    "DEPLOY_ACCOUNT"
)]
#[case::invoke(&Path::new(TEST_FILES_FOLDER).join("invoke_v3.json"), "INVOKE")]
#[tokio::test]
async fn test_routes(#[case] json_file_path: &Path, #[case] expected_response: &str) {
    let tx_json = fs::read_to_string(json_file_path).unwrap();
    let request = Request::post("/add_transaction")
        .header("content-type", "application/json")
        .body(Body::from(tx_json))
        .unwrap();

    let response = check_request(request, StatusCode::OK).await;

    assert_str_eq!(expected_response, String::from_utf8_lossy(&response));
}

// TODO(Ayelet): Replace the use of the JSON files with generated instances, then serialize these
// into JSON for testing.
#[rstest]
#[case::declare(&Path::new(TEST_FILES_FOLDER).join("declare_v3.json"), "DECLARE")]
#[case::deploy_account(
    &Path::new(TEST_FILES_FOLDER).join("deploy_account_v3.json"),
    "DEPLOY_ACCOUNT"
)]
#[case::invoke(&Path::new(TEST_FILES_FOLDER).join("invoke_v3.json"), "INVOKE")]
#[tokio::test]
async fn test_end_to_end(#[case] json_file_path: &Path, #[case] expected_response: &str) {
    // Initialize Gateway.
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = 3000;
    let gateway_config: GatewayConfig = GatewayConfig { ip, port };
    let gateway = Gateway {
        config: gateway_config.clone(),
    };

    // Setup server
    tokio::spawn(async move {
        if let Err(e) = gateway.build_server().await {
            eprintln!("Server failed: {}", e);
        }
    });

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
    let mut _mempool = Mempool;

    // // Open once the mempool is implemented.
    // let internal_transaction = create_tx_for_testing();
    // let _ = mempool.add_tx(
    //     internal_transaction,
    //     starknet_mempool::mempool::AccountState {},
    // );
    // let _tx = mempool.get_txs(1);
}

#[tokio::test]
#[should_panic]
// FIXME: Currently is_alive is not implemented, fix this once it is implemented.
async fn test_is_alive() {
    let request = Request::get("/is_alive").body(Body::empty()).unwrap();
    // Status code doesn't matter, this panics ATM.
    check_request(request, StatusCode::default()).await;
}

async fn check_request(request: Request<Body>, status_code: StatusCode) -> Bytes {
    let gateway_config = starknet_gateway::gateway::GatewayConfig {
        ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        port: 8080,
    };
    let response = app(gateway_config).oneshot(request).await.unwrap();
    assert_eq!(response.status(), status_code);

    response.into_body().collect().await.unwrap().to_bytes()
}
