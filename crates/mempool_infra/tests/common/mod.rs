use async_trait::async_trait;
use starknet_mempool_infra::component_client::ClientError;

pub(crate) type ValueA = u32;
pub(crate) type ValueB = u8;

// TODO(Tsabary): add more messages / functions to the components.

pub type ClientResult<T> = Result<T, ClientError>;

#[async_trait]
pub(crate) trait AClientTrait: Send + Sync {
    async fn a_get_value(&self) -> ClientResult<ValueA>;
}

#[async_trait]
pub(crate) trait BClientTrait: Send + Sync {
    async fn b_get_value(&self) -> ClientResult<ValueB>;
}

pub(crate) struct ComponentA {
    b: Box<dyn BClientTrait>,
}

impl ComponentA {
    pub fn new(b: Box<dyn BClientTrait>) -> Self {
        Self { b }
    }

    pub async fn a_get_value(&self) -> ValueA {
        let b_value = self.b.b_get_value().await.unwrap();
        b_value.into()
    }
}

pub(crate) struct ComponentB {
    value: ValueB,
    _a: Box<dyn AClientTrait>,
}

impl ComponentB {
    pub fn new(value: ValueB, a: Box<dyn AClientTrait>) -> Self {
        Self { value, _a: a }
    }
    pub async fn b_get_value(&self) -> ValueB {
        self.value
    }
}
