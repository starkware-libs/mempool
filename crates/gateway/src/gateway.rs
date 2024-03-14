use crate::errors::{GatewayConfigError, GatewayError};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
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
            GatewayError::ConfigError(GatewayConfigError::InvalidServerBindAddress(
                self.gateway_config.bind_address.clone(),
            ))
        })?;

        let make_service = make_service_fn(|_conn| async {
            let ctx = HandleContext {};
            Ok::<_, Infallible>(service_fn(move |req| handle_request(ctx.clone(), req)))
        });

        Server::bind(&addr)
            .serve(make_service)
            .await
            .map_err(|_| GatewayError::ServerStartError)?;

        Ok(())
    }
}

pub struct GatewayConfig {
    pub bind_address: String,
}

/// Stores routing information for request's handling.
#[derive(Clone)]
struct HandleContext {}

async fn handle_request(
    _ctx: HandleContext,
    request: RequestBody,
) -> Result<Response<Body>, GatewayError> {
    let (parts, _body) = request.into_parts();
    let response = match (parts.method, parts.uri.path()) {
        (Method::GET, "/is_alive") => is_alive(),
        _ => build_generic_response(
            StatusCode::NOT_FOUND,
            NOT_FOUND_RESPONSE.to_string(),
            GatewayError::InternalServerError,
        ),
    };
    response
}

// This function hasn't been implemented yet. It might need a HandleContext parameter to verify if
// the server is alive.
fn is_alive() -> Result<ResponseBody, GatewayError> {
    unimplemented!("Future handling should be implemented here.");
}

fn build_generic_response(
    status: StatusCode,
    body_content: String,
    error: GatewayError,
) -> Result<Response<Body>, GatewayError> {
    Response::builder()
        .status(status)
        .body(Body::from(body_content))
        .map_err(|_| error)
}
