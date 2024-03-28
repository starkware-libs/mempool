use crate::gateway::add_transaction;
use crate::gateway::handle_request;
use hyper::{Body, Request};

#[tokio::test]
async fn test_invalid_request() {
    // Create a sample GET request for an invalid path
    let request = Request::get("/some_invalid_path")
        .body(Body::empty())
        .unwrap();
    let response = handle_request(request).await.unwrap();

    assert_eq!(response.status(), 404);
    assert_eq!(
        String::from_utf8_lossy(&hyper::body::to_bytes(response.into_body()).await.unwrap()),
        "Not found."
    );
}

#[tokio::test]
async fn test_add_transaction_declare() {
    let json_str = std::fs::read_to_string("./src/json_files_for_testing/declare_v1.json")
        .expect("Failed to read JSON file");
    let body = Body::from(json_str);
    let response = add_transaction(body)
        .await
        .expect("Failed to process transaction");
    let bytes = hyper::body::to_bytes(response.into_body())
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(bytes.to_vec()).expect("Response body is not valid UTF-8");
    assert_eq!(body_str, "DECLARE");

    let json_str = std::fs::read_to_string("./src/json_files_for_testing/declare_v2.json")
        .expect("Failed to read JSON file");
    let body = Body::from(json_str);
    let response = add_transaction(body)
        .await
        .expect("Failed to process transaction");
    let bytes = hyper::body::to_bytes(response.into_body())
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(bytes.to_vec()).expect("Response body is not valid UTF-8");
    assert_eq!(body_str, "DECLARE");

    let json_str = std::fs::read_to_string("./src/json_files_for_testing/declare_v3.json")
        .expect("Failed to read JSON file");
    let body = Body::from(json_str);
    let response = add_transaction(body)
        .await
        .expect("Failed to process transaction");
    let bytes = hyper::body::to_bytes(response.into_body())
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(bytes.to_vec()).expect("Response body is not valid UTF-8");
    assert_eq!(body_str, "DECLARE");
}

#[tokio::test]
async fn test_add_transaction_deploy_account() {
    let json_str = std::fs::read_to_string("./src/json_files_for_testing/deploy_account_v1.json")
        .expect("Failed to read JSON file");
    let body = Body::from(json_str);
    let response = add_transaction(body)
        .await
        .expect("Failed to process transaction");
    let bytes = hyper::body::to_bytes(response.into_body())
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(bytes.to_vec()).expect("Response body is not valid UTF-8");
    assert_eq!(body_str, "DEPLOY_ACCOUNT");

    let json_str = std::fs::read_to_string("./src/json_files_for_testing/deploy_account_v3.json")
        .expect("Failed to read JSON file");
    let body = Body::from(json_str);
    let response = add_transaction(body)
        .await
        .expect("Failed to process transaction");
    let bytes = hyper::body::to_bytes(response.into_body())
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(bytes.to_vec()).expect("Response body is not valid UTF-8");
    assert_eq!(body_str, "DEPLOY_ACCOUNT");
}

#[tokio::test]
async fn test_add_transaction_invoke() {
    let json_str = std::fs::read_to_string("./src/json_files_for_testing/invoke_v1.json")
        .expect("Failed to read JSON file");
    let body = Body::from(json_str);
    let response = add_transaction(body)
        .await
        .expect("Failed to process transaction");
    let bytes = hyper::body::to_bytes(response.into_body())
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(bytes.to_vec()).expect("Response body is not valid UTF-8");
    assert_eq!(body_str, "INVOKE");

    let json_str = std::fs::read_to_string("./src/json_files_for_testing/invoke_v3.json")
        .expect("Failed to read JSON file");
    let body = Body::from(json_str);
    let response = add_transaction(body)
        .await
        .expect("Failed to process transaction");
    let bytes = hyper::body::to_bytes(response.into_body())
        .await
        .expect("Failed to read response body");
    let body_str = String::from_utf8(bytes.to_vec()).expect("Response body is not valid UTF-8");
    assert_eq!(body_str, "INVOKE");
}
