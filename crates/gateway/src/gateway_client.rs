use std::net::SocketAddr;

use axum::body::Body;
use hyper::StatusCode;
use reqwest::{Client, Response};
use starknet_api::external_transaction::ExternalTransaction;

use crate::errors::GatewayError;
use crate::starknet_api_test_utils::external_tx_to_json;

pub type GatewayResult<T> = Result<T, GatewayError>;

/// A test utility client for interacting with a gateway server.
pub struct GatewayClient {
    socket: SocketAddr,
    client: Client,
}

impl GatewayClient {
    pub fn new(socket: SocketAddr) -> Self {
        let client = Client::new();
        Self { socket, client }
    }

    // TODO: change from &str to proper return type once that's ready.
    pub async fn assert_add_tx_success(&self, tx: &ExternalTransaction, expected: &str) {
        let response =
            self.add_tx_with_status_check(tx, StatusCode::OK).await.bytes().await.unwrap();

        assert_eq!(response, expected)
    }

    pub async fn add_tx_with_status_check(
        &self,
        tx: &ExternalTransaction,
        expected_status_code: StatusCode,
    ) -> Response {
        let response = self.add_tx(tx).await;
        assert_eq!(response.status(), expected_status_code);

        response
    }

    // Prefer using assert_add_tx_success or other higher level methods of this client, to ensure
    // tests are boilerplate and implementation-detail free.
    pub async fn add_tx(&self, tx: &ExternalTransaction) -> Response {
        let tx_json = external_tx_to_json(tx.clone());
        self.client
            .post(format!("http://{}/add_tx", self.socket))
            .header("content-type", "application/json")
            .body(Body::from(tx_json))
            .send()
            .await
            .unwrap()
    }
}
