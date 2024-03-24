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
        let app = self.app();

        // Create a server that runs forever.
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();

        Ok(())
    }

    /// Sets up the router with the specified routes for the server.
    pub fn app(self) -> Router {
        let gateway = self.clone();
        Router::new().route("/is_alive", get(is_alive)).route(
            "/add_transaction",
            post(move |request_body: Json<ExternalTransaction>| {
                Self::add_transaction(gateway, request_body)
            }),
        )
        // TODO: when we need to configure the router, like adding banned ips, add it here via
        // `with_state`.
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
