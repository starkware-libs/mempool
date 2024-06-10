use thiserror::Error;
use tokio::sync::mpsc::{channel, Sender};

use crate::component_definitions::RequestWithResponder;

#[derive(Clone)]
pub struct ComponentClient<Request, Response>
where
    Request: Send + Sync,
    Response: Send + Sync,
{
    tx: Sender<RequestWithResponder<Request, Response>>,
}

impl<Request, Response> ComponentClient<Request, Response>
where
    Request: Send + Sync,
    Response: Send + Sync,
{
    pub fn new(tx: Sender<RequestWithResponder<Request, Response>>) -> Self {
        Self { tx }
    }

    // TODO(Tsabary, 1/5/2024): Consider implementation for messages without expected responses.

    pub async fn send(&self, request: Request) -> Response {
        let (res_tx, mut res_rx) = channel::<Response>(1);
        let request_and_res_tx = RequestWithResponder { request, tx: res_tx };
        self.tx.send(request_and_res_tx).await.expect("Outbound connection should be open.");

        res_rx.recv().await.expect("Inbound connection should be open.")
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Got an unexpected response type.")]
    UnexpectedResponse,
    #[error("Could not connect to server.")]
    ConnectionFailure,
    #[error("Could not get response from server.")]
    ResponseFailure,
}
