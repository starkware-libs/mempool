use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::external_transaction::ExternalTransaction;
use starknet_mempool_types::mempool_types::GatewayNetworkComponent;

use crate::config::{GatewayConfig, GatewayNetworkConfig};
use crate::errors::GatewayError;
use crate::stateless_transaction_validator::StatelessTransactionValidator;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult<T> = Result<T, GatewayError>;

pub struct Gateway {
    pub config: GatewayConfig,
    pub network_component: GatewayNetworkComponent,
}

#[derive(Clone)]
pub struct AppState {
    pub stateless_transaction_validator: StatelessTransactionValidator,
    /// This field uses Arc to enable shared ownership, which is necessary because
    /// `GatewayNetworkClient` supports only one receiver at a time.
    pub network_component: Arc<GatewayNetworkComponent>,
    // TODO(yael 15/5/24) add stateful_transaction_validator and state_reader_factory.
}

impl Gateway {
    pub async fn build_server(self) {
        // Parses the bind address from GatewayConfig, returning an error for invalid addresses.
        let GatewayNetworkConfig { ip, port } = self.config.network_config;
        let addr = SocketAddr::new(ip, port);
        let app = self.app();

        // Create a server that runs forever.
        axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
    }

    pub fn app(self) -> Router {
        let app_state = AppState {
            stateless_transaction_validator: StatelessTransactionValidator {
                config: self.config.stateless_transaction_validator_config,
            },
            network_component: Arc::new(self.network_component),
        };

        Router::new()
            .route("/is_alive", get(is_alive))
            .route("/add_tx", post(async_add_tx))
            .with_state(app_state)
        // TODO: when we need to configure the router, like adding banned ips, add it here via
        // `with_state`.
    }
}

async fn is_alive() -> GatewayResult<String> {
    unimplemented!("Future handling should be implemented here.");
}

async fn async_add_tx(
    State(gateway_state): State<AppState>,
    Json(tx): Json<ExternalTransaction>,
) -> GatewayResult<String> {
    tokio::task::spawn_blocking(move || add_tx(gateway_state, tx)).await?
}

fn add_tx(gateway_state: AppState, tx: ExternalTransaction) -> GatewayResult<String> {
    // TODO(Arni, 1/5/2024): Preform congestion control.

    // Perform stateless validations.
    gateway_state.stateless_transaction_validator.validate(&tx)?;

    // TODO(Yael, 1/5/2024): Preform state related validations.
    // TODO(Arni, 1/5/2024): Move transaction to mempool.

    // TODO(Arni, 1/5/2024): Produce response.
    // Send response.
    Ok(match tx {
        ExternalTransaction::Declare(_) => "DECLARE".into(),
        ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT".into(),
        ExternalTransaction::Invoke(_) => "INVOKE".into(),
    })
}
