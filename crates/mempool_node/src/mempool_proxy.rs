use crate::mempool::{AddTransactionCallType, AddTransactionReturnType, Mempool, MempoolTrait};
use async_trait::async_trait;

use tokio::sync::mpsc::{channel, Sender};
use tokio::task;

enum ProxyFunc {
    AddTransaction(AddTransactionCallType),
}

enum ProxyRetValue {
    AddTransaction(AddTransactionReturnType),
}

#[derive(Clone)]
pub struct MempoolProxy {
    tx_call: Sender<(ProxyFunc, Sender<ProxyRetValue>)>,
}

impl Default for MempoolProxy {
    fn default() -> Self {
        Self::new()
    }
}

impl MempoolProxy {
    pub fn new() -> Self {
        let (tx_call, mut rx_call) = channel::<(ProxyFunc, Sender<ProxyRetValue>)>(32);

        task::spawn(async move {
            let mut mempool = Mempool::default();
            while let Some(call) = rx_call.recv().await {
                match call {
                    (ProxyFunc::AddTransaction(tx), tx_response) => {
                        let ret_value = mempool.add_transaction(tx).await;
                        tx_response
                            .send(ProxyRetValue::AddTransaction(ret_value))
                            .await
                            .expect("Sender of the func call is expecting a return value");
                    }
                }
            }
        });

        MempoolProxy { tx_call }
    }
}

#[async_trait]
impl MempoolTrait for MempoolProxy {
    async fn add_transaction(&mut self, tx: AddTransactionCallType) -> AddTransactionReturnType {
        let (tx_response, mut rx_response) = channel(32);
        self.tx_call
            .send((ProxyFunc::AddTransaction(tx), tx_response))
            .await
            .expect("Receiver is always listening in a dedicated task");

        match rx_response.recv().await.expect(
            "Receiver of the function call always returns a return value after sending a func call",
        ) {
            ProxyRetValue::AddTransaction(ret_value) => ret_value,
        }
    }
}
