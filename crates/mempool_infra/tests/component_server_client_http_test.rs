mod common;

use std::net::IpAddr;

use async_trait::async_trait;
use common::{
    ComponentAClientTrait, ComponentARequest, ComponentAResponse, ComponentBClientTrait,
    ComponentBRequest, ComponentBResponse, ResultA, ResultB,
};
use starknet_mempool_infra::component_client::ComponentClientHttp;
use starknet_mempool_infra::component_server::ComponentServerHttp;
use tokio::task;

use crate::common::{ComponentA, ComponentB, ValueA, ValueB};

#[async_trait]
impl ComponentAClientTrait for ComponentClientHttp<ComponentA> {
    async fn a_get_value(&self) -> ResultA {
        match self.send(ComponentARequest::AGetValue).await.unwrap() {
            ComponentAResponse::Value(value) => Ok(value),
        }
    }
}

#[async_trait]
impl ComponentBClientTrait for ComponentClientHttp<ComponentB> {
    async fn b_get_value(&self) -> ResultB {
        match self.send(ComponentBRequest::BGetValue).await.unwrap() {
            ComponentBResponse::Value(value) => Ok(value),
        }
    }
}

async fn verify_response(ip_address: IpAddr, port: u16, expected_value: ValueA) {
    let a_client = ComponentClientHttp::new(ip_address, port);
    assert_eq!(a_client.a_get_value().await.unwrap(), expected_value);
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

    let component_a = ComponentA::new(Box::new(b_client));
    let component_b = ComponentB::new(setup_value, Box::new(a_client));

    let mut component_a_server = ComponentServerHttp::new(component_a, local_ip, a_port);
    let mut component_b_server = ComponentServerHttp::new(component_b, local_ip, b_port);

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
