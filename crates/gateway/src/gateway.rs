use crate::errors::GatewayError;
use crate::transaction::{ExternalTransaction, InternalTransaction};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use papyrus_common::transaction_hash::get_transaction_hash;
use papyrus_common::TransactionOptions;
use starknet_api::core::ChainId;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::task;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

const NOT_FOUND_RESPONSE: &str = "Not found.";
type RequestBody = Request<Body>;
type ResponseBody = Response<Body>;
pub type GatewayResult = Result<(), GatewayError>;

#[derive(Clone)]
pub struct Gateway {
    pub gateway_config: GatewayConfig,
}

impl Gateway {
    pub fn new(gateway_config: GatewayConfig) -> Self {
        Self { gateway_config }
    }

    pub async fn build_server(&self) -> GatewayResult {
        let addr = SocketAddr::from_str(&self.gateway_config.bind_address)
            .map_err(|_| GatewayError::ServerError)?;

        let make_service = make_service_fn(move |_conn| {
            let self_arc = Arc::new(self.clone());
            let self_clone = Arc::clone(&self_arc);
            async move {
                let ctx = HandleContext {};

                Ok::<_, Infallible>(service_fn(move |req| {
                    let ctx_clone = ctx.clone();
                    let self_clone_inner = Arc::clone(&self_clone);

                    async move { self_clone_inner.handle_request(ctx_clone, req).await }
                }))
            }
        });

        Server::bind(&addr)
            .serve(make_service)
            .await
            .map_err(|_| GatewayError::ServerError)?;

        Ok(())
    }

    async fn handle_request(
        &self,
        _ctx: HandleContext,
        request: RequestBody,
    ) -> Result<Response<Body>, GatewayError> {
        let (parts, body) = request.into_parts();
        let response = match (parts.method, parts.uri.path()) {
            (Method::GET, "/is_alive") => is_alive(),
            (Method::POST, "/add_transaction") => self.add_transaction(body).await,
            _ => Ok(Response::builder()
                .status(404)
                .body(Body::from(NOT_FOUND_RESPONSE))
                .map_err(|_| GatewayError::InternalServerError)?),
        };
        response
    }

    async fn add_transaction(&self, body: Body) -> Result<ResponseBody, GatewayError> {
        let bytes = hyper::body::to_bytes(body)
            .await
            .map_err(|_| GatewayError::InternalServerError)?;

        let self_arc = Arc::new(self.clone());
        let tx = serde_json::from_slice::<ExternalTransaction>(&bytes);
        match tx {
            Ok(transaction) => {
                let transaction_clone = transaction.clone();

                task::spawn_blocking(move || {
                    // Simulate a heavy computation
                    let chain_id = self_arc.gateway_config.chain_id.clone();
                    let _internal_tx = self_arc.convert_to_internal_tx(transaction_clone, chain_id);
                });

                let response_body = transaction.get_transaction_type();
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(response_body))
                    .map_err(|_| GatewayError::InternalServerError)?)
            }
            Err(_) => Err(GatewayError::InvalidTransactionFormat),
        }
    }

    pub fn convert_to_internal_tx(
        &self,
        tx: ExternalTransaction,
        chain_id: ChainId,
    ) -> InternalTransaction {
        let transaction = tx.get_transaction().clone();
        let only_query = TransactionOptions { only_query: false };
        let hash = get_transaction_hash(&transaction, &chain_id, &only_query).unwrap();
        InternalTransaction::new(transaction, hash)
    }
}

#[derive(Clone)]
pub struct GatewayConfig {
    pub bind_address: String,
    pub chain_id: ChainId,
}

#[derive(Clone)]
struct HandleContext {}

// This function hasn't been implemented yet. It might need a HandleContext parameter to verify if
// the server is alive.
fn is_alive() -> Result<ResponseBody, GatewayError> {
    if true {
        return Response::builder()
            .status(200)
            .body(Body::from("Server is alive"))
            .map_err(|_| GatewayError::InternalServerError);
    }
    unimplemented!("Future error handling might be implemented here.");
}
