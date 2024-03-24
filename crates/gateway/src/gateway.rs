use tokio::task;

use crate::errors::{GatewayConfigError, GatewayError};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::core::ChainId;
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
        let tx_type = match transaction_json {
            ExternalTransaction::Declare(_) => "DECLARE",
            ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT",
            ExternalTransaction::Invoke(_) => "INVOKE",
        };

        let transaction_clone = transaction_json.clone();
        let config_clone = self.config.clone();
        task::spawn_blocking(move || {
            // Simulate a heavy computation
            let _internal_tx = transaction_clone.into_internal(&config_clone.chain_id);
        });

        tx_type
    }
}

#[derive(Clone)]
pub struct GatewayConfig {
    pub bind_address: String,
    pub chain_id: ChainId,
}

async fn is_alive() -> impl IntoResponse {
    unimplemented!("Future handling should be implemented here.");
}
