use std::time::Duration;

use tokio::sync::mpsc::Receiver;
use tokio::time::timeout;

use crate::integration_test_utils::BatcherCommand;
pub struct MockBatcher {
    trigger_receiver: Receiver<BatcherCommand>,
}

impl MockBatcher {
    pub fn new(receiver: Receiver<BatcherCommand>) -> Self {
        MockBatcher { trigger_receiver: receiver }
    }

    pub async fn run(&mut self) {
        while let Some(message) =
            timeout(Duration::from_secs(5), self.trigger_receiver.recv()).await.unwrap_or(None)
        {
            match message {
                BatcherCommand::TriggerBatcher => {
                    println!("Received test request");
                }
            }
        }
    }
}
