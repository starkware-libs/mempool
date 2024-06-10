mod common;

use async_trait::async_trait;
use common::{AClient, AClientResult, BClient, BClientResult};
use starknet_mempool_infra::component_client::ComponentClient;
use starknet_mempool_infra::component_definitions::{
    ComponentRequestAndResponseSender, ComponentRequestHandler,
};
use starknet_mempool_infra::component_server::ComponentServer;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task;

use crate::common::{ComponentA, ComponentB, ValueA, ValueB};

// TODO(Tsabary): send messages from component b to component a.

pub enum RequestA {
    AGetValue,
}

pub enum ResponseA {
    Value(ValueA),
}

#[async_trait]
impl AClient for ComponentClient<RequestA, ResponseA> {
    async fn a_get_value(&self) -> AClientResult {
        let res = self.send(RequestA::AGetValue).await;
        match res {
            ResponseA::Value(value) => Ok(value),
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<RequestA, ResponseA> for ComponentA {
    async fn handle_request(&mut self, request: RequestA) -> ResponseA {
        match request {
            RequestA::AGetValue => ResponseA::Value(self.a_get_value().await),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RequestB {
    BGetValue,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResponseB {
    Value(ValueB),
}

#[async_trait]
impl BClient for ComponentClient<RequestB, ResponseB> {
    async fn b_get_value(&self) -> BClientResult {
        let res = self.send(RequestB::BGetValue).await;
        match res {
            ResponseB::Value(value) => Ok(value),
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<RequestB, ResponseB> for ComponentB {
    async fn handle_request(&mut self, request: RequestB) -> ResponseB {
        match request {
            RequestB::BGetValue => ResponseB::Value(self.b_get_value()),
        }
    }
}

async fn verify_response(
    tx_a: Sender<ComponentRequestAndResponseSender<RequestA, ResponseA>>,
    expected_value: ValueA,
) {
    let a_client = ComponentClient::new(tx_a);

    let returned_value = a_client.a_get_value().await.expect("Value should be returned");
    assert_eq!(returned_value, expected_value);
}

#[tokio::test]
async fn test_setup() {
    let setup_value: ValueB = 30;
    let expected_value: ValueA = setup_value.into();

    let (tx_a, rx_a) = channel::<ComponentRequestAndResponseSender<RequestA, ResponseA>>(32);
    let (tx_b, rx_b) = channel::<ComponentRequestAndResponseSender<RequestB, ResponseB>>(32);

    let a_client = ComponentClient::new(tx_a.clone());
    let b_client = ComponentClient::new(tx_b.clone());

    let component_a = ComponentA::new(Box::new(b_client));
    let component_b = ComponentB::new(setup_value, Box::new(a_client));

    let mut component_a_server = ComponentServer::new(component_a, rx_a);
    let mut component_b_server = ComponentServer::new(component_b, rx_b);

    task::spawn(async move {
        component_a_server.start().await;
    });

    task::spawn(async move {
        component_b_server.start().await;
    });

    verify_response(tx_a.clone(), expected_value).await;
}
