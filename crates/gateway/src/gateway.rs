use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::external_transaction::ExternalTransaction;
use std::net::{IpAddr, SocketAddr};

use crate::errors::GatewayError;
use crate::stateless_transaction_validator::{
    StatelessTransactionValidator, StatelessTransactionValidatorConfig,
};

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult = Result<(), GatewayError>;

pub struct Gateway {
    pub config: GatewayConfig,
    pub stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
}

impl Gateway {
    pub async fn build_server(self) -> GatewayResult {
        // Parses the bind address from GatewayConfig, returning an error for invalid addresses.
        let addr = SocketAddr::new(self.config.ip, self.config.port);
        let app = app(self.stateless_transaction_validator_config);

        // Create a server that runs forever.
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();

        Ok(())
    }
}

/// Sets up the router with the specified routes for the server.
pub fn app(stateless_transaction_validator_config: StatelessTransactionValidatorConfig) -> Router {
    let add_transaction_handler = |external_transaction| {
        add_transaction(stateless_transaction_validator_config, external_transaction)
    };

    Router::new()
        .route("/is_alive", get(is_alive))
        .route("/add_transaction", post(add_transaction_handler))
    // TODO: when we need to configure the router, like adding banned ips, add it here via
    // `with_state`.
}

async fn is_alive() -> impl IntoResponse {
    unimplemented!("Future handling should be implemented here.");
}

async fn add_transaction(
    stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
    Json(transaction): Json<ExternalTransaction>,
) -> impl IntoResponse {
    // TODO(Arni, 1/5/2024): Preform congestion control.

    // Perform stateless validations.
    let stateless_validator = StatelessTransactionValidator {
        config: stateless_transaction_validator_config,
    };

    if let Err(error) = stateless_validator.validate(&transaction) {
        return error.to_string();
    }

    // TODO(Arni, 1/5/2024): Descard duplications.
    // TODO(Yael, 1/5/2024): Preform state related validations.
    // TODO(Arni, 1/5/2024): Move transaction to mempool.

    // TODO(Arni, 1/5/2024): Produce response.
    // Send response.
    match transaction {
        ExternalTransaction::Declare(_) => "DECLARE".to_owned(),
        ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT".to_owned(),
        ExternalTransaction::Invoke(_) => "INVOKE".to_owned(),
    }
}

pub struct GatewayConfig {
    pub ip: IpAddr,
    pub port: u16,
}

impl GatewayConfig {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }
}
