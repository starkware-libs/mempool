use crate::errors::GatewayError;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

const NOT_FOUND_RESPONSE: &str = "Not found.";
pub type GatewayResult = Result<(), GatewayError>;

pub struct Gateway {
    pub gateway_config: GatewayConfig,
}

impl Gateway {
    pub async fn build_server(&self) -> GatewayResult {
        let addr = SocketAddr::from_str(&self.gateway_config.bind_address)
            .map_err(|_| GatewayError::ServerError)?;

        let make_service = make_service_fn(|_conn| async {
            let ctx = HandleContext {};
            Ok::<_, Infallible>(service_fn(move |req| handle_request(ctx.clone(), req)))
        });

        Server::bind(&addr)
            .serve(make_service)
            .await
            .map_err(|_| GatewayError::ServerError)?;

        Ok(())
    }
}

pub struct GatewayConfig {
    pub bind_address: String,
}

#[derive(Clone)]
struct HandleContext {}

async fn handle_request(
    _ctx: HandleContext,
    request: Request<Body>,
) -> Result<Response<Body>, GatewayError> {
    let (parts, _body) = request.into_parts();
    let response = match (parts.method, parts.uri.path()) {
        (Method::GET, "/is_alive") => is_alive(),
        _ => Ok(Response::builder()
            .status(404)
            .body(Body::from(NOT_FOUND_RESPONSE))
            .map_err(|_| GatewayError::InternalServerError)?),
    };
    response
}

// This function hasn't been implemented yet. It might need a HandleContext parameter to verify if
// the server is alive.
fn is_alive() -> Result<Response<Body>, GatewayError> {
    if true {
        return Response::builder()
            .status(200)
            .body(Body::from("Server is alive"))
            .map_err(|_| GatewayError::InternalServerError);
    }
    unimplemented!("Future error handling might be implemented here.");
}
