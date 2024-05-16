use std::net::{IpAddr, SocketAddr};

use axum::body::{Body, HttpBody};
use axum::http::Request;
use hyper::client::HttpConnector;
use hyper::{Client, Response};
use starknet_api::external_transaction::ExternalTransaction;

use crate::errors::GatewayError;
use crate::starknet_api_test_utils::external_invoke_tx_to_json;

pub type GatewayResult<T> = Result<T, GatewayError>;

// TODO: remove when struct is used in end_to_end test.
#[allow(dead_code)]
struct GatewayClient {
    ip: IpAddr,
    port: u16,
    client: Client<HttpConnector>,
}

// TODO: Use in end_to_end test when merged. Should replace `send_and_verify_transaction`
// function.
impl GatewayClient {
    fn _new(ip: IpAddr, port: u16) -> Self {
        let client = Client::new();
        Self { ip, port, client }
    }
    async fn _add_tx(&self, tx: ExternalTransaction) -> GatewayResult<String> {
        let tx_json = external_invoke_tx_to_json(tx);
        let request = Request::builder()
            .method("POST")
            .uri(format!("http://{}", SocketAddr::from((self.ip, self.port))) + "/add_transaction")
            .header("content-type", "application/json")
            .body(Body::from(tx_json))?;

        // Send a POST request with the transaction data as the body
        let response: Response<Body> = self.client.request(request).await?;

        let response_bytes = response.into_body().collect().await?.to_bytes();
        let response_string = String::from_utf8(response_bytes.to_vec())?;
        Ok(response_string)
    }
}
