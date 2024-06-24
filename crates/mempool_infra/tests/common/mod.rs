use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use starknet_mempool_infra::component_client::ClientResult;
use starknet_mempool_infra::component_server::ComponentRequestHandler;

// TODO(Tsabary): add more messages / functions to the components.

pub(crate) type ValueA = u32;
pub(crate) type ValueB = u8;

pub type ResultA = ClientResult<ValueA>;
pub type ResultB = ClientResult<ValueB>;

#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentARequest {
    AGetValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentAResponse {
    Value(ValueA),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentBRequest {
    BGetValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentBResponse {
    Value(ValueB),
}

#[async_trait]
#[allow(dead_code)] // Due to clippy flipping out for unknown reason
pub(crate) trait ComponentAClientTrait: Send + Sync {
    async fn a_get_value(&self) -> ResultA;
}

#[async_trait]
pub(crate) trait ComponentBClientTrait: Send + Sync {
    async fn b_get_value(&self) -> ResultB;
}

pub(crate) struct ComponentA {
    b: Box<dyn ComponentBClientTrait>,
}

impl ComponentA {
    pub fn new(b: Box<dyn ComponentBClientTrait>) -> Self {
        Self { b }
    }

    pub async fn a_get_value(&self) -> ValueA {
        self.b.b_get_value().await.unwrap().into()
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

pub(crate) struct ComponentB {
    value: ValueB,
    _a: Box<dyn ComponentAClientTrait>,
}

impl ComponentB {
    pub fn new(value: ValueB, a: Box<dyn ComponentAClientTrait>) -> Self {
        Self { value, _a: a }
    }

    pub fn b_get_value(&self) -> ValueB {
        self.value
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
