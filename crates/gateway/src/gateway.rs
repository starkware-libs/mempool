use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use mempool_infra::network_component::CommunicationInterface;
use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::internal_transaction::InternalTransaction;
use starknet_mempool_types::mempool_types::{
    Account, AccountState, GatewayNetworkComponent, GatewayToMempoolMessage,
};
use tokio::task;

use crate::config::GatewayConfig;
use crate::errors::GatewayError;
use crate::stateless_transaction_validator::{
    StatelessTransactionValidator, StatelessTransactionValidatorConfig,
};
use crate::utils::create_tx_for_testing;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult<T> = Result<T, GatewayError>;

pub struct Gateway {
    pub config: GatewayConfig,
    // TODO(Arni, 7/5/2024): Move the stateless transaction validator config into the gateway
    // config.
    pub stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
    pub network: GatewayNetworkComponent,
}

#[derive(Clone)]
pub struct GatewayState {
    pub stateless_transaction_validator: StatelessTransactionValidator,
}

#[derive(Clone)]
pub struct AppState {
    gateway_state: GatewayState,
    network: Arc<GatewayNetworkComponent>,
}

impl Gateway {
    pub async fn build_server(self) {
        // Parses the bind address from GatewayConfig, returning an error for invalid addresses.
        let addr = SocketAddr::new(self.config.ip, self.config.port);
        let app = self.app();

        // Create a server that runs forever.
        axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
    }

    // TODO(Arni, 7/5/2024): Change this function to accept GatewayConfig.
    /// Sets up the router with the specified routes for the server.
    pub fn app(self) -> Router {
        let gateway_state = GatewayState {
            stateless_transaction_validator: StatelessTransactionValidator {
                config: self.stateless_transaction_validator_config,
            },
        };

        // A workaround for enabling clone for state.
        let app_state = AppState { gateway_state, network: Arc::new(self.network) };

        Router::new()
            .route("/is_alive", get(is_alive))
            .route("/add_transaction", post(add_transaction))
            .with_state(app_state)
        // TODO: when we need to configure the router, like adding banned ips, add it here via
        // `with_state`.
    }
}

async fn is_alive() -> GatewayResult<String> {
    unimplemented!("Future handling should be implemented here.");
}

async fn add_transaction(
    State(app_state): State<AppState>,
    Json(transaction): Json<ExternalTransaction>,
) -> GatewayResult<String> {
    let validation_result = tokio::task::spawn_blocking(move || {
        validate_transaction(app_state.gateway_state, transaction)
    })
    .await?;

    match validation_result {
        Err(e) => Err(e),
        Ok(res) => {
            let internal_transaction = res.clone().1;
            let account = Account {
                address: ContractAddress::default(),
                state: AccountState { nonce: Nonce::default() },
            };

            let message = GatewayToMempoolMessage::AddTx(internal_transaction, account);
            task::spawn(async move {
                app_state.network.send(message).await.unwrap();
            })
            .await
            .unwrap();
            Ok(res.0)
        }
    }
}

fn validate_transaction(
    gateway_state: GatewayState,
    transaction: ExternalTransaction,
) -> GatewayResult<(String, InternalTransaction)> {
    // TODO(Arni, 1/5/2024): Preform congestion control.

    // Perform stateless validations.
    gateway_state.stateless_transaction_validator.validate(&transaction)?;

    // TODO(Yael, 1/5/2024): Preform state related validations.
    // TODO(Arni, 1/5/2024): Move transaction to mempool.

    // TODO(Arni, 1/5/2024): Produce response.
    // Send response.

    // TODO(Yael, 15/5/2024): Add tx converter.
    let internal_transaction = create_tx_for_testing();

    Ok((
        match transaction {
            ExternalTransaction::Declare(_) => "DECLARE".into(),
            ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT".into(),
            ExternalTransaction::Invoke(_) => "INVOKE".into(),
        },
        internal_transaction,
    ))
}
