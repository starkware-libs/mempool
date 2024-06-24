mod common;

use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;
use common::{
    ComponentAClientTrait, ComponentARequest, ComponentAResponse, ComponentBClientTrait,
    ComponentBRequest, ComponentBResponse, ResultA, ResultB,
};
use starknet_mempool_infra::component_client::ComponentClientHttp;
use starknet_mempool_infra::component_definitions::ComponentRequestHandler;
use starknet_mempool_infra::component_server::ComponentServerHttp;
use tokio::sync::Mutex;
use tokio::task;

use crate::common::{ComponentA, ComponentB, ValueA, ValueB};

#[async_trait]
impl ComponentAClientTrait
    for ComponentClientHttp<ComponentA, ComponentARequest, ComponentAResponse>
{
    async fn a_get_value(&self) -> ResultA {
        match self.send(ComponentARequest::AGetValue).await {
            ComponentAResponse::Value(value) => Ok(value),
        }
    }
}

#[async_trait]
impl ComponentBClientTrait
    for ComponentClientHttp<ComponentB, ComponentBRequest, ComponentBResponse>
{
    async fn b_get_value(&self) -> ResultB {
        match self.send(ComponentBRequest::BGetValue).await {
            ComponentBResponse::Value(value) => Ok(value),
        }
    }
}

async fn verify_response(ip_address: IpAddr, port: u16, expected_value: ValueA) {
    let a_client = ComponentClientHttp::new(ip_address, port);
    assert_eq!(a_client.a_get_value().await.unwrap(), expected_value);
}

#[async_trait]
impl ComponentRequestHandler<ComponentARequest, ComponentAResponse> for Arc<Mutex<ComponentA>> {
    async fn handle_request(&mut self, request: ComponentARequest) -> ComponentAResponse {
        match request {
            ComponentARequest::AGetValue => {
                ComponentAResponse::Value(self.lock().await.a_get_value().await)
            }
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<ComponentBRequest, ComponentBResponse> for Arc<Mutex<ComponentB>> {
    async fn handle_request(&mut self, request: ComponentBRequest) -> ComponentBResponse {
        match request {
            ComponentBRequest::BGetValue => {
                ComponentBResponse::Value(self.lock().await.b_get_value())
            }
        }
    }
}

#[tokio::test]
async fn test_setup() {
    let setup_value: ValueB = 90;
    let expected_value: ValueA = setup_value.into();

    let local_ip = "::1".parse().unwrap();
    let a_port = 10000;
    let b_port = 10001;

    let a_client = ComponentClientHttp::new(local_ip, a_port);
    let b_client = ComponentClientHttp::new(local_ip, b_port);

    let component_a = Arc::new(Mutex::new(ComponentA::new(Box::new(b_client))));
    let component_b = Arc::new(Mutex::new(ComponentB::new(setup_value, Box::new(a_client))));

    let component_a_server = ComponentServerHttp::new(component_a, local_ip, a_port);
    let component_b_server = ComponentServerHttp::new(component_b, local_ip, b_port);

    task::spawn(async move {
        component_a_server.start().await;
    });

    task::spawn(async move {
        component_b_server.start().await;
    });

    // Todo(uriel): Get rid of this
    task::yield_now().await;

    verify_response(local_ip, a_port, expected_value).await;
}
