use serde::Deserialize;
use starknet_api::core::ChainId;
use starknet_api::internal_transaction::InternalTransaction;
use tokio::task;

use crate::errors::GatewayError;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::external_transaction::ExternalTransaction;
use std::net::{IpAddr, SocketAddr};

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
        let addr = SocketAddr::new(self.config.ip, self.config.port);
        let app = app();

        // Create a server that runs forever.
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();

        Ok(())
    }
}

/// Sets up the router with the specified routes for the server.
pub fn app() -> Router {
    Router::new()
        .route("/is_alive", get(is_alive))
        .route("/add_transaction", post(add_transaction))
    // TODO: when we need to configure the router, like adding banned ips, add it here via
    // `with_state`.
}

async fn is_alive() -> impl IntoResponse {
    unimplemented!("Future handling should be implemented here.");
}

async fn add_transaction(Json(input): Json<GatewayTransactionInput>) -> impl IntoResponse {
    let transaction_clone = input.tx.clone();
    let handle = task::spawn_blocking(move || {
        // Simulate a heavy computation
        transaction_clone.into_internal(&input.chain_id)
    });

    let internal_tx = handle.await.unwrap();
    let internal_tx = match internal_tx {
        InternalTransaction::Declare(tx) => tx.tx_hash,
        InternalTransaction::DeployAccount(tx) => tx.tx_hash,
        InternalTransaction::Invoke(tx) => tx.tx_hash,
    };
    internal_tx.to_string().into_response()
}

#[derive(Clone, Debug)]
pub struct GatewayConfig {
    pub ip: IpAddr,
    pub port: u16,
}

impl GatewayConfig {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }
}

#[derive(Deserialize)]
pub struct GatewayTransactionInput {
    chain_id: ChainId,
    tx: ExternalTransaction,
}
