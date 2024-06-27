mod common;

use async_trait::async_trait;
use common::{ComponentAClientTrait, ComponentBClientTrait, ResultA, ResultB};
use starknet_mempool_infra::component_client::{ClientError, ComponentClient};
use starknet_mempool_infra::component_definitions::{
    ComponentRequestAndResponseSender, ComponentRequestHandler,
};
use starknet_mempool_infra::component_server::ComponentServer;
use tokio::sync::mpsc::channel;
use tokio::task::{self, AbortHandle};

use crate::common::{ComponentA, ComponentB, ValueA, ValueB};

// TODO(Tsabary): send messages from component b to component a.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComponentARequest {
    AGetValue,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComponentAResponse {
    Value(ValueA),
}

#[async_trait]
impl ComponentAClientTrait for ComponentClient<ComponentARequest, ComponentAResponse> {
    async fn a_get_value(&self) -> ResultA {
        let res = self.send(ComponentARequest::AGetValue).await?;
        match res {
            ComponentAResponse::Value(value) => Ok(value),
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<ComponentARequest, ComponentAResponse> for ComponentA {
    async fn handle_request(&mut self, request: ComponentARequest) -> ComponentAResponse {
        match request {
            ComponentARequest::AGetValue => ComponentAResponse::Value(self.a_get_value().await),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComponentBRequest {
    BGetValue,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComponentBResponse {
    Value(ValueB),
}

#[async_trait]
impl ComponentBClientTrait for ComponentClient<ComponentBRequest, ComponentBResponse> {
    async fn b_get_value(&self) -> ResultB {
        let res = self.send(ComponentBRequest::BGetValue).await?;
        match res {
            ComponentBResponse::Value(value) => Ok(value),
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<ComponentBRequest, ComponentBResponse> for ComponentB {
    async fn handle_request(&mut self, request: ComponentBRequest) -> ComponentBResponse {
        match request {
            ComponentBRequest::BGetValue => ComponentBResponse::Value(self.b_get_value()),
        }
    }
}

async fn verify_response(
    a_client: &ComponentClient<ComponentARequest, ComponentAResponse>,
    expected_value: ValueA,
) {
    assert_eq!(a_client.a_get_value().await, Ok(expected_value));
}

async fn verify_error(
    a_client: ComponentClient<ComponentARequest, ComponentAResponse>,
    b_client: ComponentClient<ComponentBRequest, ComponentBResponse>,
    abort_handle_a: AbortHandle,
    abort_handle_b: AbortHandle,
) {
    // Aborting a task takse place when the task is active again, which in the following two cases
    // after the main task yields.

    // Case 1: Not waiting for the abortion to finish, making it fail after sending the request
    abort_handle_a.abort();
    let response = a_client.a_get_value().await;
    assert_eq!(response, Err(ClientError::ChannelNoResponse));

    // Case 2: Let the abortion finish first, this way it will fail while sending the request
    abort_handle_b.abort();
    task::yield_now().await;
    let response = b_client.b_get_value().await;
    assert_eq!(response, Err(ClientError::ChannelSendError));
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

    let component_a = ComponentA::new(Box::new(b_client.clone()));
    let component_b = ComponentB::new(setup_value, Box::new(a_client.clone()));

    let mut component_a_server = ComponentServer::new(component_a, rx_a);
    let mut component_b_server = ComponentServer::new(component_b, rx_b);

    let abort_handle_a = task::spawn(async move {
        component_a_server.start().await;
    })
    .abort_handle();

    let abort_handle_b = task::spawn(async move {
        component_b_server.start().await;
    })
    .abort_handle();

    verify_response(&a_client, expected_value).await;
    verify_error(a_client, b_client, abort_handle_a, abort_handle_b).await;
}
