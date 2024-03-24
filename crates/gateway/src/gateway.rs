use starknet_api::core::ChainId;
use starknet_api::internal_transaction::InternalTransaction;
use tokio::task;

use crate::errors::{GatewayConfigError, GatewayError};
use crate::GatewayConfig;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::external_transaction::ExternalTransaction;
use std::net::SocketAddr;
use std::str::FromStr;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult = Result<(), GatewayError>;

#[derive(Clone)]
pub struct Gateway {
    pub config: GatewayConfig,
    pub chain_id: ChainId,
}

impl Gateway {
    pub async fn build_server(self) -> GatewayResult {
        // Parses the bind address from GatewayConfig, returning an error for invalid addresses.
        let addr = SocketAddr::from_str(&self.config.bind_address).map_err(|_| {
            GatewayConfigError::InvalidServerBindAddress(self.config.bind_address.clone())
        })?;
        let gateway = self.clone();

        // Sets up the router with the specified routes for the server.
        let app = Router::new().route("/is_alive", get(is_alive)).route(
            "/add_transaction",
            post(move |request_body: Json<ExternalTransaction>| {
                Self::add_transaction(gateway, request_body)
            }),
        );

        // Create a server that runs forever.
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();

        Ok(())
    }

    async fn add_transaction(
        self,
        Json(transaction_json): Json<ExternalTransaction>,
    ) -> impl IntoResponse {
        let transaction_clone = transaction_json.clone();
        let handle = task::spawn_blocking(move || {
            // Simulate a heavy computation
            transaction_clone.into_internal(&self.chain_id)
        });

        let internal_tx = handle.await.unwrap();
        match internal_tx {
            InternalTransaction::Declare(tx) => tx.tx_hash.to_string().into_response(),
            InternalTransaction::DeployAccount(tx) => tx.tx_hash.to_string().into_response(),
            InternalTransaction::Invoke(tx) => tx.tx_hash.to_string().into_response(),
        }
    }
}

async fn is_alive() -> impl IntoResponse {
    unimplemented!("Future handling should be implemented here.");
}
