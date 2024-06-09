use std::clone::Clone;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::mempool_types::{Account, MempoolClient, MempoolInput};

use crate::config::{GatewayConfig, GatewayNetworkConfig};
use crate::errors::{GatewayError, GatewayRunError};
use crate::starknet_api_test_utils::get_sender_address;
use crate::state_reader::StateReaderFactory;
use crate::stateful_transaction_validator::StatefulTransactionValidator;
use crate::stateless_transaction_validator::StatelessTransactionValidator;
use crate::utils::{external_tx_to_account_tx, external_tx_to_thin_tx, get_tx_hash};

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult<T> = Result<T, GatewayError>;

pub struct Gateway {
    config: GatewayConfig,
    app_state: AppState,
}

#[derive(Clone)]
pub struct AppState {
    pub stateless_tx_validator: StatelessTransactionValidator,
    pub stateful_tx_validator: Arc<StatefulTransactionValidator>,
    pub state_reader_factory: Arc<dyn StateReaderFactory>,
    pub mempool_client: Arc<dyn MempoolClient>,
}

impl Gateway {
    pub fn new(
        config: GatewayConfig,
        state_reader_factory: Arc<dyn StateReaderFactory>,
        mempool_client: Arc<dyn MempoolClient>,
    ) -> Self {
        let app_state = AppState {
            stateless_tx_validator: StatelessTransactionValidator {
                config: config.stateless_tx_validator_config.clone(),
            },
            stateful_tx_validator: Arc::new(StatefulTransactionValidator {
                config: config.stateful_tx_validator_config.clone(),
            }),
            state_reader_factory,
            mempool_client,
        };
        Gateway { config, app_state }
    }

    pub async fn run(self) -> Result<(), GatewayRunError> {
        // Parses the bind address from GatewayConfig, returning an error for invalid addresses.
        let GatewayNetworkConfig { ip, port } = self.config.network_config;
        let addr = SocketAddr::new(ip, port);
        let app = self.app();

        // Create a server that runs forever.
        Ok(axum::Server::bind(&addr).serve(app.into_make_service()).await?)
    }

    pub fn app(self) -> Router {
        Router::new()
            .route("/is_alive", get(is_alive))
            .route("/add_tx", post(add_tx))
            .with_state(self.app_state)
        // TODO: when we need to configure the router, like adding banned ips, add it here via
        // `with_state`.
    }
}

// Gateway handlers.

async fn is_alive() -> GatewayResult<String> {
    unimplemented!("Future handling should be implemented here.");
}

async fn add_tx(
    State(app_state): State<AppState>,
    Json(tx): Json<ExternalTransaction>,
) -> GatewayResult<Json<TransactionHash>> {
    let mempool_input = tokio::task::spawn_blocking(move || {
        process_tx(
            app_state.stateless_tx_validator,
            app_state.stateful_tx_validator.as_ref(),
            app_state.state_reader_factory.as_ref(),
            tx,
        )
    })
    .await??;

    let tx_hash = mempool_input.tx.tx_hash;

    app_state
        .mempool_client
        .add_tx(mempool_input)
        .await
        .map_err(|e| GatewayError::MessageSendError(e.to_string()))?;
    // TODO: Also return `ContractAddress` for deploy and `ClassHash` for Declare.
    Ok(Json(tx_hash))
}

fn process_tx(
    stateless_tx_validator: StatelessTransactionValidator,
    stateful_tx_validator: &StatefulTransactionValidator,
    state_reader_factory: &dyn StateReaderFactory,
    tx: ExternalTransaction,
) -> GatewayResult<MempoolInput> {
    // TODO(Arni, 1/5/2024): Perform congestion control.

    // Perform stateless validations.
    stateless_tx_validator.validate(&tx)?;

    // Convert to internal transaction.
    let account_tx =
        external_tx_to_account_tx(&tx, None, &stateful_tx_validator.config.chain_info.chain_id)?;
    let tx_hash = get_tx_hash(&account_tx);
    let thin_tx = external_tx_to_thin_tx(&tx, tx_hash);

    // TODO(Yael, 19/5/2024): pass the relevant class_info and deploy_account_hash.
    stateful_tx_validator.run_validate(state_reader_factory, account_tx, None)?;

    Ok(MempoolInput {
        tx: thin_tx,
        account: Account { sender_address: get_sender_address(&tx), ..Default::default() },
    })
}
