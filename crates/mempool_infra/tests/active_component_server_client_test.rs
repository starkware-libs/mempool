use std::future::pending;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use starknet_mempool_infra::component_client::definitions::ClientResult;
use starknet_mempool_infra::component_client::local_component_client::LocalComponentClient;
use starknet_mempool_infra::component_definitions::{
    ComponentRequestAndResponseSender, ComponentRequestHandler,
};
use starknet_mempool_infra::component_runner::{ComponentStartError, ComponentStarter};
use starknet_mempool_infra::component_server::definitions::ComponentServerStarter;
use starknet_mempool_infra::component_server::empty_component_server::EmptyServer;
use starknet_mempool_infra::component_server::local_component_server::LocalActiveComponentServer;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{sleep, Duration};

pub type ValueC = i64;
pub(crate) type ResultC = ClientResult<ValueC>;

#[derive(Debug, Clone)]
pub(crate) struct ComponentC {
    value: Arc<Mutex<ValueC>>,
}

impl ComponentC {
    pub fn new(value: ValueC) -> Self {
        Self { value: Arc::new(Mutex::new(value)) }
    }

    pub async fn c_get_value(&self) -> ValueC {
        *self.value.lock().await
    }

    pub async fn c_increment_value(&self) -> ValueC {
        let mut value = self.value.lock().await;
        *value += 1;
        *value
    }
}

#[async_trait]
impl ComponentStarter for ComponentC {
    async fn start(&mut self) -> Result<(), ComponentStartError> {
        for _ in 0..10 {
            sleep(Duration::from_millis(200)).await;
            self.c_increment_value().await;
        }
        let val = self.c_get_value().await;
        assert!(val >= 10);

        let () = pending().await;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentCRequest {
    CIncValue,
    CGetValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentCResponse {
    Value(ValueC),
}

#[async_trait]
pub(crate) trait ComponentCClientTrait: Send + Sync {
    async fn c_increment_value(&self) -> ResultC;
    async fn c_get_value(&self) -> ResultC;
}

pub(crate) struct ComponentD {
    c: Box<dyn ComponentCClientTrait>,
}

impl ComponentD {
    pub fn new(c: Box<dyn ComponentCClientTrait>) -> Self {
        Self { c }
    }

    pub async fn d_increment_value(&self) -> ValueC {
        self.c.c_increment_value().await.unwrap()
    }

    pub async fn d_get_value(&self) -> ValueC {
        self.c.c_get_value().await.unwrap()
    }
}

#[async_trait]
impl ComponentStarter for ComponentD {
    async fn start(&mut self) -> Result<(), ComponentStartError> {
        for i in 0..4 {
            sleep(Duration::from_millis(100 * (i + 1))).await;
            self.d_increment_value().await;
        }
        let val = self.d_get_value().await;
        assert!(val >= 4);
        let () = pending().await;
        Ok(())
    }
}

#[async_trait]
impl ComponentCClientTrait for LocalComponentClient<ComponentCRequest, ComponentCResponse> {
    async fn c_increment_value(&self) -> ResultC {
        let res = self.send(ComponentCRequest::CIncValue).await;
        match res {
            ComponentCResponse::Value(value) => Ok(value),
        }
    }

    async fn c_get_value(&self) -> ResultC {
        let res = self.send(ComponentCRequest::CGetValue).await;
        match res {
            ComponentCResponse::Value(value) => Ok(value),
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<ComponentCRequest, ComponentCResponse> for ComponentC {
    async fn handle_request(&mut self, request: ComponentCRequest) -> ComponentCResponse {
        match request {
            ComponentCRequest::CGetValue => ComponentCResponse::Value(self.c_get_value().await),
            ComponentCRequest::CIncValue => {
                ComponentCResponse::Value(self.c_increment_value().await)
            }
        }
    }
}

async fn verify_response_c_d(
    tx_c: Sender<ComponentRequestAndResponseSender<ComponentCRequest, ComponentCResponse>>,
    expected_value: ValueC,
) {
    let c_client = LocalComponentClient::new(tx_c);
    assert_eq!(c_client.c_get_value().await.unwrap(), expected_value);
}

#[tokio::test]
async fn test_setup_c_d() {
    let setup_value: ValueC = 0;
    let expected_value: ValueC = 14;

    let (tx_c, rx_c) =
        channel::<ComponentRequestAndResponseSender<ComponentCRequest, ComponentCResponse>>(32);

    let c_client = LocalComponentClient::new(tx_c.clone());

    let component_c = ComponentC::new(setup_value);
    let component_d = ComponentD::new(Box::new(c_client));

    let mut component_c_server = LocalActiveComponentServer::new(component_c, rx_c);
    let mut component_d_server = EmptyServer::new(component_d);

    task::spawn(async move {
        component_c_server.start().await;
    });

    task::spawn(async move {
        component_d_server.start().await;
    });

    // Wait for the components to finish incrementing of the ComponentC::value.
    sleep(Duration::from_millis(2100)).await;

    verify_response_c_d(tx_c.clone(), expected_value).await;
}
