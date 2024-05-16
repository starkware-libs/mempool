use std::clone::Clone;
use std::marker::{PhantomData, Send, Sync};
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::external_transaction::ExternalTransaction;
use starknet_mempool_types::mempool_types::GatewayNetworkComponent;

use crate::config::{GatewayNetworkConfig, StatelessTransactionValidatorConfig};
use crate::errors::GatewayError;
use crate::state_reader::{MempoolStateReader, StateReaderFactory};
use crate::stateful_transaction_validator::{
    StatefulTransactionValidator, StatefulTransactionValidatorConfig,
};
use crate::stateless_transaction_validator::StatelessTransactionValidator;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult<T> = Result<T, GatewayError>;

pub struct Gateway<T, S>
where
    T: StateReaderFactory<S>,
    S: MempoolStateReader,
{
    pub network_config: GatewayNetworkConfig,
    // TODO(Arni, 7/5/2024): Move the stateless transaction validator config into the gateway
    // config.
    pub stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
    pub stateful_transaction_validator_config: StatefulTransactionValidatorConfig,
    pub network_component: GatewayNetworkComponent,
    pub state_reader_factory: T,
    pub _phantom: PhantomData<S>,
}

#[derive(Clone)]
pub struct AppState<T, S>
where
    T: StateReaderFactory<S>,
    S: MempoolStateReader,
{
    pub stateless_transaction_validator: StatelessTransactionValidator,
    pub stateful_transaction_validator: StatefulTransactionValidator,
    /// This field uses Arc to enable shared ownership, which is necessary because
    /// `GatewayNetworkClient` supports only one receiver at a time.
    pub network_component: Arc<GatewayNetworkComponent>,
    pub state_reader_factory: T,
    _phantom: PhantomData<S>,
}

impl<T, S> Gateway<T, S>
where
    T: StateReaderFactory<S> + Send + Sync + Clone + 'static,
    S: MempoolStateReader + Send + Sync + Clone + 'static,
{
    pub async fn build_server(self) {
        // Parses the bind address from GatewayConfig, returning an error for invalid addresses.
        let addr = SocketAddr::new(self.network_config.ip, self.network_config.port);
        let app = self.app();

        // Create a server that runs forever.
        axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
    }

    pub fn app(self) -> Router {
        let app_state = AppState {
            stateless_transaction_validator: StatelessTransactionValidator {
                config: self.stateless_transaction_validator_config,
            },
            stateful_transaction_validator: StatefulTransactionValidator {
                config: self.stateful_transaction_validator_config,
            },
            network_component: Arc::new(self.network_component),
            state_reader_factory: self.state_reader_factory,
            _phantom: Default::default(),
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

async fn async_add_transaction<T, S>(
    State(gateway_state): State<AppState<T, S>>,
    Json(transaction): Json<ExternalTransaction>,
) -> GatewayResult<String>
where
    T: StateReaderFactory<S> + std::marker::Send + 'static,
    S: MempoolStateReader + std::marker::Send + 'static,
{
    tokio::task::spawn_blocking(move || add_transaction(gateway_state, transaction)).await?
}

fn add_transaction<T, S>(
    gateway_state: AppState<T, S>,
    transaction: ExternalTransaction,
) -> GatewayResult<String>
where
    T: StateReaderFactory<S>,
    S: MempoolStateReader,
{
    // TODO(Arni, 1/5/2024): Preform congestion control.

    // Perform stateless validations.
    gateway_state.stateless_transaction_validator.validate(&transaction)?;

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
