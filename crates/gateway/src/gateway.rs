use crate::errors::{GatewayConfigError, GatewayError};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use starknet_api::external_transaction::ExternalTransaction;
use std::net::SocketAddr;
use std::str::FromStr;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult = Result<(), GatewayError>;

pub struct Gateway {
    pub gateway_config: GatewayConfig,
}

impl Gateway {
    pub async fn build_server(&self) -> GatewayResult {
        let addr = SocketAddr::from_str(&self.gateway_config.bind_address).map_err(|_| {
            GatewayConfigError::InvalidServerBindAddress(self.gateway_config.bind_address.clone())
        })?;

        let app = Router::new()
            .route("/is_alive", get(is_alive))
            .route("/add_transaction", post(add_transaction));

        let _ = axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .map_err(|_| GatewayError::ServerStartError);

        Ok(())
    }
}

pub struct GatewayConfig {
    pub bind_address: String,
}

async fn is_alive() -> impl IntoResponse {
    unimplemented!("Future handling should be implemented here.");
}

async fn add_transaction(Json(transaction_json): Json<ExternalTransaction>) -> impl IntoResponse {
    match transaction_json {
        ExternalTransaction::Declare(_) => "DECLARE",
        ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT",
        ExternalTransaction::Invoke(_) => "INVOKE",
    }
}
