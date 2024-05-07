use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::Mutex;
use tokio::task;

pub type AddTransactionCallType = u32;
pub type AddTransactionReturnType = usize;

#[derive(PartialEq, Debug)]
pub struct GetTransactionReturnType {
    pub value: usize,
}

#[async_trait]
pub trait AddTransactionTrait {
    async fn add_transaction(&self, tx: AddTransactionCallType) -> AddTransactionReturnType;
}

#[async_trait]
pub trait GetTransactionTrait {
    async fn get_transaction(&self) -> GetTransactionReturnType;
}

#[derive(Default)]
pub struct CompA {
    transactions: Mutex<Vec<u32>>,
}

#[async_trait]
impl AddTransactionTrait for CompA {
    async fn add_transaction(&self, tx: AddTransactionCallType) -> AddTransactionReturnType {
        let mut guarded_transactions = self.transactions.lock().await;
        guarded_transactions.push(tx);
        guarded_transactions.len()
    }
}

#[async_trait]
impl GetTransactionTrait for CompA {
    async fn get_transaction(&self) -> GetTransactionReturnType {
        let guarded_transactions = self.transactions.lock().await;
        GetTransactionReturnType {
            value: guarded_transactions.len(),
        }
    }
}

enum ProxyFunc {
    AddTransaction(AddTransactionCallType),
    GetTransaction(),
}

enum ProxyRetValue {
    AddTransaction(AddTransactionReturnType),
    GetTransaction(GetTransactionReturnType),
}

#[derive(Clone)]
pub struct CompAProxy {
    tx_call: Sender<(ProxyFunc, Sender<ProxyRetValue>)>,
}

impl Default for CompAProxy {
    fn default() -> Self {
        let (tx_call, mut rx_call) = channel::<(ProxyFunc, Sender<ProxyRetValue>)>(32);

        task::spawn(async move {
            let comp_a = Arc::new(CompA::default());
            while let Some(call) = rx_call.recv().await {
                match call {
                    (ProxyFunc::AddTransaction(tx), tx_response) => {
                        let comp_a = comp_a.clone();
                        task::spawn(async move {
                            let ret_value = comp_a.add_transaction(tx).await;
                            tx_response
                                .send(ProxyRetValue::AddTransaction(ret_value))
                                .await
                                .expect("Receiver should be listening.");
                        });
                    }
                    (ProxyFunc::GetTransaction(), tx_response) => {
                        let comp_a = comp_a.clone();
                        task::spawn(async move {
                            let ret_value = comp_a.get_transaction().await;
                            tx_response
                                .send(ProxyRetValue::GetTransaction(ret_value))
                                .await
                                .expect("Receiver should be listening.");
                        });
                    }
                }
            }
        });
        Self { tx_call }
    }
}

#[async_trait]
impl AddTransactionTrait for CompAProxy {
    async fn add_transaction(&self, tx: AddTransactionCallType) -> AddTransactionReturnType {
        let (tx_response, mut rx_response) = channel(32);
        self.tx_call
            .send((ProxyFunc::AddTransaction(tx), tx_response))
            .await
            .expect("Receiver should be listening.");

        match rx_response
            .recv()
            .await
            .expect("Sender should be responding.")
        {
            ProxyRetValue::AddTransaction(ret_value) => ret_value,
            _ => todo!(),
        }
    }
}

#[async_trait]
impl GetTransactionTrait for CompAProxy {
    async fn get_transaction(&self) -> GetTransactionReturnType {
        let (tx_response, mut rx_response) = channel(32);
        self.tx_call
            .send((ProxyFunc::GetTransaction(), tx_response))
            .await
            .expect("Receiver should be listening.");

        match rx_response
            .recv()
            .await
            .expect("Sender should be responding.")
        {
            ProxyRetValue::GetTransaction(ret_value) => ret_value,
            _ => todo!(),
        }
    }
}
