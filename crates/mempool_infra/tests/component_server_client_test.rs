mod common;

use async_trait::async_trait;
use common::{
    ComponentAClientTrait, ComponentARequest, ComponentAResponse, ComponentBClientTrait,
    ComponentBRequest, ComponentBResponse, ResultA, ResultB,
};
use starknet_mempool_infra::component_client::ComponentClient;
use starknet_mempool_infra::component_definitions::ComponentRequestAndResponseSender;
use starknet_mempool_infra::component_server::ComponentServer;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task;

use crate::common::{ComponentA, ComponentB, ValueA, ValueB};

// TODO(Tsabary): send messages from component b to component a.

#[async_trait]
impl ComponentAClientTrait for ComponentClient<ComponentARequest, ComponentAResponse> {
    async fn a_get_value(&self) -> ResultA {
        let res = self.send(ComponentARequest::AGetValue).await.unwrap();
        match res {
            ComponentAResponse::Value(value) => Ok(value),
        }
    }
}

#[async_trait]
impl ComponentBClientTrait for ComponentClient<ComponentBRequest, ComponentBResponse> {
    async fn b_get_value(&self) -> ResultB {
        let res = self.send(ComponentBRequest::BGetValue).await.unwrap();
        match res {
            ComponentBResponse::Value(value) => Ok(value),
        }
    }
}

async fn verify_response(
    tx_a: Sender<ComponentRequestAndResponseSender<ComponentARequest, ComponentAResponse>>,
    expected_value: ValueA,
) {
    let (tx_a_main, mut rx_a_main) = channel::<ComponentAResponse>(1);

    let request_and_res_tx: ComponentRequestAndResponseSender<
        ComponentARequest,
        ComponentAResponse,
    > = ComponentRequestAndResponseSender { request: ComponentARequest::AGetValue, tx: tx_a_main };

    tx_a.send(request_and_res_tx).await.unwrap();

    let res = rx_a_main.recv().await.unwrap();
    match res {
        ComponentAResponse::Value(value) => {
            assert_eq!(value, expected_value);
        }
    }
}

#[tokio::test]
async fn test_setup() {
    let setup_value: ValueB = 30;
    let expected_value: ValueA = setup_value.into();

    let (tx_a, rx_a) =
        channel::<ComponentRequestAndResponseSender<ComponentARequest, ComponentAResponse>>(32);
    let (tx_b, rx_b) =
        channel::<ComponentRequestAndResponseSender<ComponentBRequest, ComponentBResponse>>(32);

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
