use async_trait::async_trait;
use tokio::sync::mpsc::{error::SendError, Receiver, Sender};
use tokio::sync::Mutex;

#[async_trait]
pub trait CommunicationInterface<S, R> {
    async fn send(&self, message: S) -> Result<(), SendError<S>>;
    async fn recv(&self) -> Option<R>;
}

pub struct NetworkComponent<S, R> {
    tx: Mutex<Sender<S>>,
    rx: Mutex<Receiver<R>>,
}

impl<S, R> NetworkComponent<S, R> {
    pub fn new(tx: Sender<S>, rx: Receiver<R>) -> Self {
        Self {
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }
}

#[async_trait]
impl<S, R> CommunicationInterface<S, R> for NetworkComponent<S, R>
where
    S: Send + Sync,
    R: Send + Sync,
{
    async fn send(&self, message: S) -> Result<(), SendError<S>> {
        self.tx.lock().await.send(message).await
    }

    async fn recv(&self) -> Option<R> {
        self.rx.lock().await.recv().await
    }
}
