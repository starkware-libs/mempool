use std::net::IpAddr;

use bincode::serialize;
use hyper::{Body, Client, Request as HyperRequest, Uri};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc::{channel, Sender};

use crate::component_definitions::ComponentRequestAndResponseSender;

pub struct ComponentClient<Request, Response>
where
    Request: Send + Sync,
    Response: Send + Sync,
{
    tx: Sender<ComponentRequestAndResponseSender<Request, Response>>,
}

impl<Request, Response> ComponentClient<Request, Response>
where
    Request: Send + Sync,
    Response: Send + Sync,
{
    pub fn new(tx: Sender<ComponentRequestAndResponseSender<Request, Response>>) -> Self {
        Self { tx }
    }

    // TODO(Tsabary, 1/5/2024): Consider implementation for messages without expected responses.

    pub async fn send(&self, request: Request) -> Response {
        let (res_tx, mut res_rx) = channel::<Response>(1);
        let request_and_res_tx = ComponentRequestAndResponseSender { request, tx: res_tx };
        self.tx.send(request_and_res_tx).await.expect("Outbound connection should be open.");

        res_rx.recv().await.expect("Inbound connection should be open.")
    }
}

// Can't derive because derive forces the generics to also be `Clone`, which we prefer not to do
// since it'll require transactions to be cloneable.
impl<Request, Response> Clone for ComponentClient<Request, Response>
where
    Request: Send + Sync,
    Response: Send + Sync,
{
    fn clone(&self) -> Self {
        Self { tx: self.tx.clone() }
    }
}

pub struct ComponentClientHttp<Component> {
    uri: Uri,
    _component: std::marker::PhantomData<Component>,
}

impl<Component> ComponentClientHttp<Component> {
    pub fn new(ip_address: IpAddr, port: u16) -> Self {
        let uri = match ip_address {
            IpAddr::V4(ip_address) => format!("http://{}:{}/", ip_address, port).parse().unwrap(),
            IpAddr::V6(ip_address) => format!("http://[{}]:{}/", ip_address, port).parse().unwrap(),
        };
        Self { uri, _component: Default::default() }
    }

    pub async fn send<Request, Response>(&self, component_request: Request) -> Response
    where
        Request: Serialize,
        Response: for<'a> Deserialize<'a>,
    {
        let http_request = HyperRequest::post(self.uri.clone())
            .header("Content-Type", "application/octet-stream")
            .body(Body::from(
                serialize(&component_request).expect("Request serialization should succeed"),
            ))
            .expect("Request builidng should succeed");

        // TODO(uriel): Add configuration to control number of retries
        let http_response =
            Client::new().request(http_request).await.expect("Could not connect to server");
        let body_bytes = hyper::body::to_bytes(http_response.into_body())
            .await
            .expect("Could not get response from server");

        bincode::deserialize(&body_bytes).expect("Response deserialization should succeed")
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Got an unexpected response type.")]
    UnexpectedResponse,
}

pub type ClientResult<T> = Result<T, ClientError>;
