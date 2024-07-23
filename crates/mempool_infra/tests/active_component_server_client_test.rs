use std::future::pending;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use starknet_mempool_infra::component_client::definitions::{ClientError, ClientResult};
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
use tokio::time::{Duration, Instant};

type ValueC = i64;
type ResultC = ClientResult<ValueC>;

#[derive(Debug, Clone)]
pub(crate) struct ComponentC {
    value: Arc<Mutex<ValueC>>,
    test_end_count: Arc<Mutex<ValueC>>,
    test_loop_count: ValueC,
}

impl ComponentC {
    pub fn new(value: ValueC, test_loop_count: ValueC) -> Self {
        Self { value: Arc::new(Mutex::new(value)), test_end_count: Arc::new(Mutex::new(0)), test_loop_count }
    }

    pub async fn c_get_value(&self) -> ValueC {
        *self.value.lock().await
    }

    pub async fn c_increment_value(&self) {
        *self.value.lock().await += 1;
    }

    pub async fn increment_test_end_count(&self) {
        *self.test_end_count.lock().await += 1;
    }

    pub async fn get_test_end_count(&self) -> ValueC {
        *self.test_end_count.lock().await
    }
}

#[async_trait]
impl ComponentStarter for ComponentC {
    async fn start(&mut self) -> Result<(), ComponentStartError> {
        for _ in 0..self.test_loop_count {
            task::yield_now().await;
            self.c_increment_value().await;
        }
        let val = self.c_get_value().await;
        assert!(val >= self.test_loop_count);
        self.increment_test_end_count().await;
        let () = pending().await;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentCRequest {
    CIncValue,
    CGetValue,
    CTestEnd,
    CGetTestEndCount,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentCResponse {
    CIncValue,
    CGetValue(ValueC),
    CTestEnd,
    CGetTestEndCount(ValueC),
}

#[async_trait]
pub(crate) trait ComponentCClientTrait: Send + Sync {
    async fn c_inc_value(&self) -> ClientResult<()>;
    async fn c_get_value(&self) -> ResultC;
    async fn c_test_end(&self) -> ClientResult<()>;
    async fn c_get_test_end_count(&self) -> ResultC;
}

pub(crate) struct ComponentD {
    c: Box<dyn ComponentCClientTrait>,
    test_loop_count: ValueC,
}

impl ComponentD {
    pub fn new(c: Box<dyn ComponentCClientTrait>, test_loop_count: ValueC) -> Self {
        Self { c, test_loop_count }
    }

    pub async fn d_increment_value(&self) {
        self.c.c_inc_value().await.unwrap()
    }

    pub async fn d_get_value(&self) -> ValueC {
        self.c.c_get_value().await.unwrap()
    }

    pub async fn d_send_test_end(&self) {
        self.c.c_test_end().await.unwrap()
    }
}

#[async_trait]
impl ComponentStarter for ComponentD {
    async fn start(&mut self) -> Result<(), ComponentStartError> {
        for _ in 0..self.test_loop_count {
            self.d_increment_value().await;
        }
        let val = self.d_get_value().await;
        assert!(val >= self.test_loop_count);
        self.d_send_test_end().await;
        let () = pending().await;
        Ok(())
    }
}

#[async_trait]
impl ComponentCClientTrait for LocalComponentClient<ComponentCRequest, ComponentCResponse> {
    async fn c_inc_value(&self) -> ClientResult<()> {
        let res = self.send(ComponentCRequest::CIncValue).await;
        match res {
            ComponentCResponse::CIncValue => Ok(()),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    async fn c_get_value(&self) -> ResultC {
        let res = self.send(ComponentCRequest::CGetValue).await;
        match res {
            ComponentCResponse::CGetValue(value) => Ok(value),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    async fn c_test_end(&self) -> ClientResult<()> {
        let res = self.send(ComponentCRequest::CTestEnd).await;
        match res {
            ComponentCResponse::CTestEnd => Ok(()),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    async fn c_get_test_end_count(&self) -> ResultC {
        let res = self.send(ComponentCRequest::CGetTestEndCount).await;
        match res {
            ComponentCResponse::CGetTestEndCount(value) => Ok(value),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<ComponentCRequest, ComponentCResponse> for ComponentC {
    async fn handle_request(&mut self, request: ComponentCRequest) -> ComponentCResponse {
        match request {
            ComponentCRequest::CGetValue => ComponentCResponse::CGetValue(self.c_get_value().await),
            ComponentCRequest::CIncValue => {
                self.c_increment_value().await;
                ComponentCResponse::CIncValue
            }
            ComponentCRequest::CTestEnd => {
                self.increment_test_end_count().await;
                ComponentCResponse::CTestEnd
            }
            ComponentCRequest::CGetTestEndCount => {
                ComponentCResponse::CGetTestEndCount(self.get_test_end_count().await)
            }
        }
    }
}

async fn wait_and_verify_response(
    tx_c: Sender<ComponentRequestAndResponseSender<ComponentCRequest, ComponentCResponse>>,
    expected_value: ValueC,
    time_out_in_sec: u64,
) {
    let c_client = LocalComponentClient::new(tx_c);

    let start_time = Instant::now();
    let time_out = Duration::from_secs(time_out_in_sec);
    let mut test_end_count = 0;

    while start_time.elapsed() < time_out && test_end_count < 2 {
        test_end_count = c_client.c_get_test_end_count().await.unwrap();
        task::yield_now().await;
    }
    assert_eq!(test_end_count, 2);
    assert_eq!(c_client.c_get_value().await.unwrap(), expected_value);
}

#[tokio::test]
async fn test_setup_c_d() {
    let setup_value: ValueC = 0;
    let test_loop_count: ValueC = 1024; 
    let expected_value = test_loop_count * 2;
    let time_out_in_sec = 2;

    let (tx_c, rx_c) =
        channel::<ComponentRequestAndResponseSender<ComponentCRequest, ComponentCResponse>>(32);

    let c_client = LocalComponentClient::new(tx_c.clone());

    let component_c = ComponentC::new(setup_value, test_loop_count);
    let component_d = ComponentD::new(Box::new(c_client), test_loop_count);

    let mut component_c_server = LocalActiveComponentServer::new(component_c, rx_c);
    let mut component_d_server = EmptyServer::new(component_d);

    task::spawn(async move {
        component_c_server.start().await;
    });

    task::spawn(async move {
        component_d_server.start().await;
    });

    // Wait for the components to finish incrementing of the ComponentC::value and verify it.
    wait_and_verify_response(tx_c.clone(), expected_value, time_out_in_sec).await;
}
