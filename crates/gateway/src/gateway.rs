use crate::errors::GatewayError;
use crate::transaction::ExternalTransaction;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

const NOT_FOUND_RESPONSE: &str = "Not found.";
type RequestBody = Request<Body>;
type ResponseBody = Response<Body>;

pub struct GatewayConfig {
    pub config: String,
}

pub struct Gateway {
    pub gateway_config: GatewayConfig,
}

impl Gateway {
    pub fn new(gateway_config: GatewayConfig) -> Self {
        Self { gateway_config }
    }

    pub async fn build_server(&self) -> Result<(), GatewayError> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

        let make_svc = make_service_fn(|_conn| async {
            Ok::<_, Infallible>(service_fn(Self::handle_request))
        });

        let _ = Server::bind(&addr)
            .serve(make_svc)
            .await
            .map_err(|_| GatewayError::ServerError);

        Ok(())
    }

    async fn handle_request(request: RequestBody) -> Result<ResponseBody, GatewayError> {
        let (parts, body) = request.into_parts();
        let response = match (parts.method, parts.uri.path()) {
            (Method::GET, "/is_alive") => is_alive(),
            (Method::POST, "/add_transaction") => add_transaction(body).await,
            _ => Ok(Response::builder()
                .status(404)
                .body(Body::from(NOT_FOUND_RESPONSE))
                .map_err(|_| GatewayError::InternalServerError)?),
        };
        response
    }
}

fn is_alive() -> Result<ResponseBody, GatewayError> {
    Response::builder()
        .status(200)
        .body(Body::from("Server is alive"))
        .map_err(|_| GatewayError::InternalServerError)
}

async fn add_transaction(body: Body) -> Result<ResponseBody, GatewayError> {
    let bytes = hyper::body::to_bytes(body)
        .await
        .map_err(|_| GatewayError::InternalServerError)?;

    match serde_json::from_slice::<ExternalTransaction>(&bytes) {
        Ok(transaction) => {
            let response_body = transaction.get_transaction_type();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(response_body))
                .map_err(|_| GatewayError::InternalServerError)?)
        }
        Err(_) => Err(GatewayError::InvalidTransactionFormat),
    }
}
