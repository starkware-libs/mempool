use thiserror::Error;
use tokio::sync::mpsc::{channel, Sender};
use tonic::transport::Error as TonicError;
use tonic::Status as TonicStatus;

use crate::component_definitions::ComponentRequestAndResponseSender;

#[derive(Clone)]
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

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Got an unexpected response type.")]
    UnexpectedResponse,
    #[error("Failed to connect to the server")]
    ConnectionFailure(TonicError),
    #[error("Failed to get a response from the server")]
    ResponseFailure(TonicStatus),
}
