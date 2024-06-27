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

    pub async fn send(&self, request: Request) -> ClientResult<Response> {
        let (res_tx, mut res_rx) = channel::<Response>(1);
        let request_and_res_tx = ComponentRequestAndResponseSender { request, tx: res_tx };
        self.tx.send(request_and_res_tx).await.map_err(|_e| ClientError::ChannelSendError)?;

        res_rx.recv().await.ok_or(ClientError::ChannelNoResponse)
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

#[derive(Debug, Error, PartialEq)]
pub enum ClientError {
    #[error("Got an unexpected response type.")]
    UnexpectedResponse,
    #[error("Could not send request to server.")]
    ChannelSendError,
    #[error("Not received a response from server.")]
    ChannelNoResponse,
}

pub type ClientResult<Response> = Result<Response, ClientError>;
