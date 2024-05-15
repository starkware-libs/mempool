use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use mempool_infra::network_component::CommunicationInterface;
use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::external_transaction::ExternalTransaction;
use starknet_api::transaction::{Tip, TransactionHash};
use starknet_mempool_types::mempool_types::{
    Account, AccountState, GatewayNetworkComponent, GatewayToMempoolMessage, ThinTransaction,
};

use crate::config::{GatewayNetworkConfig, StatelessTransactionValidatorConfig};
use crate::errors::GatewayError;
use crate::stateless_transaction_validator::StatelessTransactionValidator;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult<T> = Result<T, GatewayError>;

pub struct Gateway {
    pub network_config: GatewayNetworkConfig,
    // TODO(Arni, 7/5/2024): Move the stateless transaction validator config into the gateway
    // config.
    pub stateless_transaction_validator_config: StatelessTransactionValidatorConfig,
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
            network_component: Arc::new(self.network_component),
        };

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
    let result = tokio::task::spawn_blocking(move || {
        validate_and_pre_process(app_state.stateless_transaction_validator, transaction)
    })
    .await??;

    let (response, thin_transaction, account) = result;
    let message = GatewayToMempoolMessage::AddTx(thin_transaction, account);
    app_state.network_component.send(message).await.unwrap();
    Ok(response)
}

fn validate_and_pre_process(
    validator: StatelessTransactionValidator,
    transaction: ExternalTransaction,
) -> GatewayResult<(String, ThinTransaction, Account)> {
    // TODO(Arni, 1/5/2024): Preform congestion control.

    // Perform stateless validations.
    validator.validate(&transaction)?;

    // TODO(Yael, 1/5/2024): Preform state related validations.

    // TODO(Arni, 1/5/2024): Produce response.
    // Send response.

    // TODO(Yael, 15/5/2024): Add tx converter.

    Ok((
        match transaction {
            ExternalTransaction::Declare(_) => "DECLARE".into(),
            ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT".into(),
            ExternalTransaction::Invoke(_) => "INVOKE".into(),
        },
        ThinTransaction {
            tip: Tip::default(),
            nonce: Nonce::default(),
            contract_address: ContractAddress::default(),
            tx_hash: TransactionHash::default(),
        },
        Account {
            address: ContractAddress::default(),
            state: AccountState { nonce: Nonce::default() },
        },
    ))
}
