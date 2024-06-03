use std::net::SocketAddr;

use axum::body::Body;
use reqwest::{Client, Response};
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::transaction::TransactionHash;

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

    pub async fn assert_add_tx_success(&self, tx: &ExternalTransaction) -> TransactionHash {
        let response = self.add_tx(tx).await;
        assert!(response.status().is_success());

        response.json().await.unwrap()
    }

    // TODO: implement when usage eventually arises.
    pub async fn assert_add_tx_error(&self, _tx: &ExternalTransaction) -> GatewayError {
        todo!()
    }

    // Prefer using assert_add_tx_success or other higher level methods of this client, to ensure
    // tests are boilerplate and implementation-detail free.
    pub async fn add_tx(&self, tx: &ExternalTransaction) -> Response {
        let tx_json = external_tx_to_json(tx);
        self.client
            .post(format!("http://{}/add_tx", self.socket))
            .header("content-type", "application/json")
            .body(Body::from(tx_json))
            .send()
            .await
            .unwrap()
    }
}
