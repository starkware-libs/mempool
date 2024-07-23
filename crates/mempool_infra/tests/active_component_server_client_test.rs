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
use tokio::time::{sleep, Duration};

type ValueC = i64;
type ResultC = ClientResult<ValueC>;

#[derive(Debug, Clone)]
struct ComponentC {
    test_counter: Arc<Mutex<ValueC>>,
    max_iterations: ValueC,
    c_test_ended: Arc<Mutex<bool>>,
    d_test_ended: Arc<Mutex<bool>>,
}

impl ComponentC {
    pub fn new(init_value: ValueC, max_iterations: ValueC) -> Self {
        Self {
            test_counter: Arc::new(Mutex::new(init_value)),
            max_iterations,
            c_test_ended: Arc::new(Mutex::new(false)),
            d_test_ended: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn c_get_test_counter(&self) -> ValueC {
        *self.test_counter.lock().await
    }

    pub async fn c_increment_test_counter(&self) {
        *self.test_counter.lock().await += 1;
    }

    pub async fn c_set_c_test_end(&self) {
        *self.c_test_ended.lock().await = true;
    }

    pub async fn c_set_d_test_end(&self) {
        *self.d_test_ended.lock().await = true;
    }

    pub async fn c_check_test_ended(&self) -> bool {
        let c_test_ended = *self.c_test_ended.lock().await;
        c_test_ended && *self.d_test_ended.lock().await
    }
}

#[async_trait]
impl ComponentStarter for ComponentC {
    async fn start(&mut self) -> Result<(), ComponentStartError> {
        for _ in 0..self.max_iterations {
            self.c_increment_test_counter().await;
        }
        let val = self.c_get_test_counter().await;
        assert!(val >= self.max_iterations);
        self.c_set_c_test_end().await;

        // Mimicing real start function that should not return.
        let () = pending().await;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentCRequest {
    CIncValue,
    CGetValue,
    CSetDTestEnd,
    CTestEndCheck,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentCResponse {
    CIncValue,
    CGetValue(ValueC),
    CSetDTestEnd,
    CTestEndCheck(bool),
}

#[async_trait]
trait ComponentCClientTrait: Send + Sync {
    async fn c_inc_value(&self) -> ClientResult<()>;
    async fn c_get_value(&self) -> ResultC;
    async fn c_set_d_test_end(&self) -> ClientResult<()>;
    async fn c_test_end_check(&self) -> ClientResult<bool>;
}

struct ComponentD {
    c: Box<dyn ComponentCClientTrait>,
    max_iterations: ValueC,
}

impl ComponentD {
    pub fn new(c: Box<dyn ComponentCClientTrait>, max_iterations: ValueC) -> Self {
        Self { c, max_iterations }
    }

    pub async fn d_increment_value(&self) {
        self.c.c_inc_value().await.unwrap()
    }

    pub async fn d_get_value(&self) -> ValueC {
        self.c.c_get_value().await.unwrap()
    }

    pub async fn d_send_test_end(&self) {
        self.c.c_set_d_test_end().await.unwrap()
    }
}

#[async_trait]
impl ComponentStarter for ComponentD {
    async fn start(&mut self) -> Result<(), ComponentStartError> {
        for _ in 0..self.max_iterations {
            self.d_increment_value().await;
        }
        let val = self.d_get_value().await;
        assert!(val >= self.max_iterations);
        self.d_send_test_end().await;

        // Mimicing real start function that should not return.
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

    async fn c_set_d_test_end(&self) -> ClientResult<()> {
        let res = self.send(ComponentCRequest::CSetDTestEnd).await;
        match res {
            ComponentCResponse::CSetDTestEnd => Ok(()),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    async fn c_test_end_check(&self) -> ClientResult<bool> {
        let res = self.send(ComponentCRequest::CTestEndCheck).await;
        match res {
            ComponentCResponse::CTestEndCheck(value) => Ok(value),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<ComponentCRequest, ComponentCResponse> for ComponentC {
    async fn handle_request(&mut self, request: ComponentCRequest) -> ComponentCResponse {
        match request {
            ComponentCRequest::CGetValue => {
                ComponentCResponse::CGetValue(self.c_get_test_counter().await)
            }
            ComponentCRequest::CIncValue => {
                self.c_increment_test_counter().await;
                ComponentCResponse::CIncValue
            }
            ComponentCRequest::CSetDTestEnd => {
                self.c_set_d_test_end().await;
                ComponentCResponse::CSetDTestEnd
            }
            ComponentCRequest::CTestEndCheck => {
                ComponentCResponse::CTestEndCheck(self.c_check_test_ended().await)
            }
        }
    }
}

async fn wait_and_verify_response(
    tx_c: Sender<ComponentRequestAndResponseSender<ComponentCRequest, ComponentCResponse>>,
    expected_value: ValueC,
) {
    let c_client = LocalComponentClient::new(tx_c);

    let delay = Duration::from_millis(1);
    let mut test_ended = false;

    while !test_ended {
        test_ended = c_client.c_test_end_check().await.unwrap();
        sleep(delay).await; // Lower CPU usage.
    }
    assert_eq!(c_client.c_get_value().await.unwrap(), expected_value);
}

#[tokio::test]
async fn test_setup_c_d() {
    let setup_value: ValueC = 0;
    let max_iterations: ValueC = 1024;
    let expected_value = max_iterations * 2;

    let (tx_c, rx_c) =
        channel::<ComponentRequestAndResponseSender<ComponentCRequest, ComponentCResponse>>(32);

    let c_client = LocalComponentClient::new(tx_c.clone());

    let component_c = ComponentC::new(setup_value, max_iterations);
    let component_d = ComponentD::new(Box::new(c_client), max_iterations);

    let mut component_c_server = LocalActiveComponentServer::new(component_c, rx_c);
    let mut component_d_server = EmptyServer::new(component_d);

    task::spawn(async move {
        component_c_server.start().await;
    });

    task::spawn(async move {
        component_d_server.start().await;
    });

    // Wait for the components to finish incrementing of the ComponentC::value and verify it.
    wait_and_verify_response(tx_c.clone(), expected_value).await;
}
