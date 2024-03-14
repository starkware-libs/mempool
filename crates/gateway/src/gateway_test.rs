use crate::gateway::handle_request;
use crate::gateway::Gateway;
use crate::gateway::GatewayConfig;
use crate::gateway::HandleContext;
use hyper::{Body, Request};
use tokio::time::{delay_for, Duration};

#[tokio::test]
async fn test_invalid_request() {
    // Create a sample GET request for an invalid path
    let request = Request::get("/some_invalid_path")
        .body(Body::empty())
        .unwrap();
    let response = handle_request(HandleContext {}, request).await.unwrap();

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
