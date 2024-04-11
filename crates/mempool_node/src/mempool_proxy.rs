use crate::mempool::{AddTransactionCallType, AddTransactionReturnType, Mempool, MempoolTrait};
use async_trait::async_trait;
use std::sync::Arc;

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task;

enum ProxyFunc {
    AddTransaction(u32),
}

enum ProxyRetValue {
    AddTransaction(bool),
}

pub struct MempoolProxy {
    tx_call: Sender<ProxyFunc>,
    rx_ret_value: Receiver<ProxyRetValue>,
}

impl MempoolProxy {
    pub fn new(mempool: Arc<Mutex<Mempool>>) -> Self {
        let (tx_call, mut rx_call) = channel(32);
        let (tx_ret_value, rx_ret_value) = channel(32);

        task::spawn(async move {
            while let Some(call) = rx_call.recv().await {
                match call {
                    ProxyFunc::AddTransaction(trx) => {
                        let ret_value = mempool.lock().await.add_transaction(trx).await;
                        tx_ret_value
                            .send(ProxyRetValue::AddTransaction(ret_value))
                            .await
                            .unwrap();
                    }
                }
            }
        });

        MempoolProxy {
            tx_call,
            rx_ret_value,
        }
    }
}

#[async_trait]
impl MempoolTrait for MempoolProxy {
    async fn add_transaction(&mut self, trx: AddTransactionCallType) -> AddTransactionReturnType {
        self.tx_call
            .send(ProxyFunc::AddTransaction(trx))
            .await
            .unwrap();
        match self.rx_ret_value.recv().await {
            Some(ProxyRetValue::AddTransaction(b)) => b,
            None => false,
        }
    }
}
