use tokio::sync::mpsc::{channel, Sender};

use crate::component_server::MessageAndResponseSender;

#[derive(Clone)]
pub struct ComponentClient<M, R>
where
    M: Send + Sync,
    R: Send + Sync,
{
    tx: Sender<MessageAndResponseSender<M, R>>,
}

// todo remove the static constraints

impl<M, R> ComponentClient<M, R>
where
    M: Send + Sync,
    R: Send + Sync,
{
    pub fn new(tx: Sender<MessageAndResponseSender<M, R>>) -> Self {
        Self { tx }
    }

    pub async fn send(&self, message: M) -> R {
        let (res_tx, mut res_rx) = channel::<R>(1);
        let message_and_res_tx = MessageAndResponseSender {
            message,
            tx: res_tx,
        };
        self.tx.send(message_and_res_tx).await.unwrap();

        res_rx.recv().await.unwrap()
    }
    // todo impl for no-response types
}
