use crate::errors::{GatewayConfigError, GatewayError};
use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use starknet_api::external_transaction::ExternalTransaction;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

const NOT_FOUND_RESPONSE: &str = "Not found.";
type RequestBody = Request<Body>;
type ResponseBody = Response<Body>;
pub type GatewayResult = Result<(), GatewayError>;

pub struct Gateway {
    pub gateway_config: GatewayConfig,
}

impl Gateway {
    pub async fn build_server(&self) -> GatewayResult {
        let addr = SocketAddr::from_str(&self.gateway_config.bind_address).map_err(|_| {
            GatewayConfigError::InvalidServerBindAddress(self.gateway_config.bind_address.clone())
        })?;

        let make_service =
            make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

        Server::bind(&addr).serve(make_service).await?;
        Server::bind(&addr).serve(make_service).await?;

        Ok(())
    }
}

pub struct GatewayConfig {
    pub bind_address: String,
}

async fn handle_request(request: RequestBody) -> Result<Response<Body>, GatewayError> {
    let (parts, body) = request.into_parts();
    let response = match (parts.method, parts.uri.path()) {
        (Method::GET, "/is_alive") => is_alive(),
        (Method::POST, "/add_transaction") => add_transaction(body).await,
        _ => response(StatusCode::NOT_FOUND, NOT_FOUND_RESPONSE.to_string()),
    };
    response
}

fn is_alive() -> Result<ResponseBody, GatewayError> {
    unimplemented!("Future handling should be implemented here.");
}

// TODO(Ayelet): Consider using axum instead of Hyper.
async fn add_transaction(body: Body) -> Result<Response<Body>, GatewayError> {
    let bytes = to_bytes(body).await?;
    let deserialized_transaction = serde_json::from_slice::<ExternalTransaction>(&bytes)
        .map_err(|_| GatewayError::InvalidTransactionFormat);

    match deserialized_transaction {
        Ok(transaction) => {
            let tx_type = match transaction {
                ExternalTransaction::Declare(_) => "DECLARE",
                ExternalTransaction::DeployAccount(_) => "DEPLOY_ACCOUNT",
                ExternalTransaction::Invoke(_) => "INVOKE",
            };
            response(StatusCode::OK, tx_type.to_string())
        }
        Err(_) => response(
            StatusCode::BAD_REQUEST,
            "Invalid transaction format.".to_string(),
        ),
    }
}

fn response(status: StatusCode, body_content: String) -> Result<Response<Body>, GatewayError> {
    Ok(Response::builder()
        .status(status)
        .body(Body::from(body_content))?)
}
