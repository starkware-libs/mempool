use axum::body::{Body, Bytes, HttpBody};
use axum::http::{Request, StatusCode};
use pretty_assertions::assert_str_eq;
use rstest::rstest;
use starknet_api::core::ChainId;
use starknet_gateway::gateway;
use std::fs;
use std::net::IpAddr;
use std::path::Path;
use tower::ServiceExt;

const TEST_FILES_FOLDER: &str = "./tests/fixtures";

// TODO(Ayelet): Replace the use of the JSON files with generated instances, then serialize these
// into JSON for testing.
#[rstest]
#[case::declare(
    &Path::new(TEST_FILES_FOLDER).join("declare_v3.json"),
    "0x03822dbc50d129064b16e3ed3ff1af2cb34cdb15f202ea6c5ec98f1cc0190ede"
)]
#[case::deploy_account(
    &Path::new(TEST_FILES_FOLDER).join("deploy_account_v3.json"),
    "0x0274837d32404e10c2aba2c782854abd692eaed9e4d46676f5611c90de6979f9"
)]
#[case::invoke(
    &Path::new(TEST_FILES_FOLDER).join("invoke_v3.json"),
    "0x06401db149315bdd1370b04c911d2e83789017574665744afa257bc1abb00308"
)]
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

#[tokio::test]
#[should_panic]
// FIXME: Currently is_alive is not implemented, fix this once it is implemented.
async fn test_is_alive() {
    let request = Request::get("/is_alive").body(Body::empty()).unwrap();
    // Status code doesn't matter, this panics ATM.
    check_request(request, StatusCode::default()).await;
}

async fn check_request(request: Request<Body>, status_code: StatusCode) -> Bytes {
    let gateway = gateway::Gateway {
        config: gateway::GatewayConfig {
            ip: IpAddr::from([0, 0, 0, 0]),
            port: 8080,
        },
        chain_id: ChainId("SN_TEST".to_string()),
    };
    let response = gateway.app().oneshot(request).await.unwrap();
    assert_eq!(response.status(), status_code);

    response.into_body().collect().await.unwrap().to_bytes()
}
