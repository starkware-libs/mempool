use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(PartialEq, Debug)]
pub enum NetworkComponentError {
    SendFailure,
}

#[async_trait]
pub trait CommunicationInterface {
    type SendType;
    type ReceiveType;
    async fn send(&mut self, message: Self::SendType) -> Result<(), NetworkComponentError>;
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
    async fn send(&mut self, message: Self::SendType) -> Result<(), NetworkComponentError> {
        if self.tx.send(message).await.is_err() {
            return Err(NetworkComponentError::SendFailure);
        }

        Ok(())
    }

    async fn recv(&mut self) -> Option<Self::ReceiveType> {
        self.rx.recv().await
    }
}
