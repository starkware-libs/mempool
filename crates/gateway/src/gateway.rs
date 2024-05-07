use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::external_transaction::ExternalTransaction;
use std::net::SocketAddr;

use crate::config::GatewayConfig;
use crate::errors::GatewayError;
use crate::stateless_transaction_validator::StatelessTransactionValidator;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult<T> = Result<T, GatewayError>;

pub struct Gateway {
    pub config: GatewayConfig,
}

#[derive(Clone)]
pub struct GatewayState {
    pub stateless_transaction_validator: StatelessTransactionValidator,
}

impl Gateway {
    pub async fn build_server(self) {
        // Parses the bind address from GatewayConfig, returning an error for invalid addresses.
        let addr = SocketAddr::new(self.config.ip, self.config.port);
        let app = app(self.config);

        // Create a server that runs forever.
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

/// Sets up the router with the specified routes for the server.
pub fn app(config: GatewayConfig) -> Router {
    let gateway_state = GatewayState {
        stateless_transaction_validator: StatelessTransactionValidator {
            config: config.stateless_transaction_validator_config,
        },
    };

    Router::new()
        .route("/is_alive", get(is_alive))
        .route("/add_transaction", post(add_transaction))
        .with_state(gateway_state)
    // TODO: when we need to configure the router, like adding banned ips, add it here via
    // `with_state`.
}

async fn is_alive() -> GatewayResult<String> {
    unimplemented!("Future handling should be implemented here.");
}

async fn add_transaction(
    State(gateway_state): State<GatewayState>,
    Json(transaction): Json<ExternalTransaction>,
) -> GatewayResult<String> {
    // TODO(Arni, 1/5/2024): Preform congestion control.

    // Perform stateless validations.
    gateway_state
        .stateless_transaction_validator
        .validate(&transaction)?;

    // TODO(Yael, 1/5/2024): Preform state related validations.
    // TODO(Arni, 1/5/2024): Move transaction to mempool.

    // TODO(Arni, 1/5/2024): Produce response.
    // Send response.
    Ok(match transaction {
        ExternalTransaction::Declare(_) => "DECLARE".into(),
        ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT".into(),
        ExternalTransaction::Invoke(_) => "INVOKE".into(),
    })
}
