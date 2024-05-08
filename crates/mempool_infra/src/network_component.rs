use async_trait::async_trait;
use tokio::sync::mpsc::{error::SendError, Receiver, Sender};

#[async_trait]
pub trait CommunicationInterface {
    type SendType;
    type ReceiveType;
    async fn send(&self, message: Self::SendType) -> Result<(), SendError<Self::SendType>>;
    async fn recv(&mut self) -> Option<Self::ReceiveType>;
}

pub struct NetworkComponent<S, R> {
    tx: Sender<S>,
    rx: Receiver<R>,
}

impl<S, R> NetworkComponent<S, R> {
    pub fn new(tx: Sender<S>, rx: Receiver<R>) -> Self {
        Self { tx, rx }
    }
}

#[async_trait]
impl<S, R> CommunicationInterface for NetworkComponent<S, R>
where
    S: Send + Sync,
    R: Send + Sync,
{
    type SendType = S;
    type ReceiveType = R;
    async fn send(&self, message: Self::SendType) -> Result<(), SendError<Self::SendType>> {
        self.tx.send(message).await
    }

    async fn recv(&mut self) -> Option<Self::ReceiveType> {
        self.rx.recv().await
    }
}
