use crate::errors::GatewayError;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

const NOT_FOUND_RESPONSE: &str = "Not found.";

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

    async fn handle_request(request: Request<Body>) -> Result<Response<Body>, GatewayError> {
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
}

fn is_alive() -> Result<Response<Body>, GatewayError> {
    Response::builder()
        .status(200)
        .body(Body::from("Server is alive"))
        .map_err(|_| GatewayError::InternalServerError)
}
