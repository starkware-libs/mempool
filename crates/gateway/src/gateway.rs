use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::external_transaction::ExternalTransaction;
use std::net::{IpAddr, SocketAddr};

use crate::errors::GatewayError;
use crate::stateless_transaction_validator::{
    StatelessTransactionValidator, StatelessTransactionValidatorConfig,
};

pub enum GatewayResponse {
    ServerBuilt,
    TransactionAccepted(&'static str),
}

impl IntoResponse for GatewayResponse {
    fn into_response(self) -> Response {
        match self {
            GatewayResponse::ServerBuilt => StatusCode::OK.into_response(),
            GatewayResponse::TransactionAccepted(response) => {
                (StatusCode::OK, response).into_response()
            }
        }
    }
}

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult = Result<GatewayResponse, GatewayError>;

pub struct Gateway {
    pub config: GatewayConfig,
}

impl Gateway {
    pub async fn build_server(self) -> GatewayResult {
        // Parses the bind address from GatewayConfig, returning an error for invalid addresses.
        let addr = SocketAddr::new(self.config.ip, self.config.port);
        let app = app(self.config);

        // Create a server that runs forever.
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();

        Ok(GatewayResponse::ServerBuilt)
    }
}

/// Sets up the router with the specified routes for the server.
pub fn app(config: GatewayConfig) -> Router {
    Router::new()
        .route("/is_alive", get(is_alive))
        .route("/add_transaction", post(add_transaction))
        .with_state(config)
    // TODO: when we need to configure the router, like adding banned ips, add it here via
    // `with_state`.
}

async fn is_alive() -> impl IntoResponse {
    unimplemented!("Future handling should be implemented here.");
}

async fn add_transaction(
    State(config): State<GatewayConfig>,
    Json(transaction): Json<ExternalTransaction>,
) -> GatewayResult {
    // TODO(Arni, 1/5/2024): Preform congestion control.

    // Perform stateless validations.
    let stateless_validator = StatelessTransactionValidator {
        config: config.stateless_transaction_validator_config,
    };

    stateless_validator.validate(&transaction)?;

    // TODO(Arni, 1/5/2024): Descard duplications.
    // TODO(Yael, 1/5/2024): Preform state related validations.
    // TODO(Arni, 1/5/2024): Move transaction to mempool.

    // TODO(Arni, 1/5/2024): Produce response.
    // Send response.
    let positive_flow_response = match transaction {
        ExternalTransaction::Declare(_) => "DECLARE",
        ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT",
        ExternalTransaction::Invoke(_) => "INVOKE",
    };
    Ok(GatewayResponse::TransactionAccepted(positive_flow_response))
}

#[derive(Clone)]
pub struct GatewayConfig {
    pub ip: IpAddr,
    pub port: u16,

    pub stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
}
