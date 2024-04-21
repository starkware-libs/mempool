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

pub struct Gateway {
    pub config: GatewayConfig,
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
        .route("/add_transaction", post(async_add_transaction))
    // TODO: when we need to configure the router, like adding banned ips, add it here via
    // `with_state`.
}

async fn is_alive() -> impl IntoResponse {
    unimplemented!("Future handling should be implemented here.");
}

fn add_transaction(transaction: ExternalTransaction) -> impl IntoResponse {
    match transaction {
        ExternalTransaction::Declare(_) => "DECLARE",
        ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT",
        ExternalTransaction::Invoke(_) => "INVOKE",
    }
}

async fn async_add_transaction(Json(transaction): Json<ExternalTransaction>) -> impl IntoResponse {
    tokio::task::spawn_blocking(move || add_transaction(transaction))
        .await
        .map_err(|_| GatewayError::InternalServerError)
}

pub struct GatewayConfig {
    pub ip: IpAddr,
    pub port: u16,
}
