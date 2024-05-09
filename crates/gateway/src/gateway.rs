use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::external_transaction::ExternalTransaction;
use std::net::SocketAddr;
use std::sync::Arc;

use starknet_mempool_types::mempool_types::GatewayNetworkComponent;

use crate::config::GatewayConfig;

use crate::errors::GatewayError;
use crate::stateless_transaction_validator::{
    StatelessTransactionValidator, StatelessTransactionValidatorConfig,
};

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult<T> = Result<T, GatewayError>;

pub struct Gateway {
    pub config: GatewayConfig,
    // TODO(Arni, 7/5/2024): Move the stateless transaction validator config into the gateway config.
    pub stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
    pub network: GatewayNetworkComponent,
}

#[derive(Clone)]
pub struct AppState {
    pub stateless_transaction_validator: StatelessTransactionValidator,
    pub network: Arc<GatewayNetworkComponent>,
}

impl Gateway {
    pub async fn build_server(self) {
        // Parses the bind address from GatewayConfig, returning an error for invalid addresses.
        let addr = SocketAddr::new(self.config.ip, self.config.port);
        let app = self.app();

        // Create a server that runs forever.
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    // TODO(Arni, 7/5/2024): Change this function to accept GatewayConfig.
    /// Sets up the router with the specified routes for the server.
    pub fn app(self) -> Router {
        let app_state = AppState {
            stateless_transaction_validator: StatelessTransactionValidator {
                config: self.stateless_transaction_validator_config,
            },
            network: Arc::new(self.network),
        };

        Router::new()
            .route("/is_alive", get(is_alive))
            .route("/add_transaction", post(async_add_transaction))
            .with_state(app_state)
        // TODO: when we need to configure the router, like adding banned ips, add it here via
        // `with_state`.
    }
}

async fn is_alive() -> GatewayResult<String> {
    unimplemented!("Future handling should be implemented here.");
}

async fn async_add_transaction(
    State(gateway_state): State<AppState>,
    Json(transaction): Json<ExternalTransaction>,
) -> GatewayResult<String> {
    tokio::task::spawn_blocking(move || add_transaction(gateway_state, transaction)).await?
}

fn add_transaction(
    gateway_state: AppState,
    transaction: ExternalTransaction,
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
