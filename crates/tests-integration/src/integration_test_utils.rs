use std::net::SocketAddr;

use axum::body::Body;
use reqwest::{Client, Response};
use starknet_api::rpc_transaction::RPCTransaction;
use starknet_api::transaction::TransactionHash;
use starknet_gateway::errors::GatewayError;
use starknet_gateway::starknet_api_test_utils::external_tx_to_json;

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

    pub async fn assert_add_tx_success(&self, tx: &RPCTransaction) -> TransactionHash {
        let response = self.add_tx(tx).await;
        assert!(response.status().is_success());

        response.json().await.unwrap()
    }

    // TODO: implement when usage eventually arises.
    pub fn assert_add_tx_error(&self, _tx: &RPCTransaction) -> GatewayError {
        todo!()
    }

    // Prefer using assert_add_tx_success or other higher level methods of this client, to ensure
    // tests are boilerplate and implementation-detail free.
    pub async fn add_tx(&self, tx: &RPCTransaction) -> Response {
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
