use async_trait::async_trait;
use starknet_mempool_infra::component_client::ClientError;

pub(crate) type ValueA = u32;
pub(crate) type ValueB = u8;

// TODO(Tsabary): add more messages / functions to the components.

#[async_trait]
pub(crate) trait ClientATrait: Send + Sync {
    async fn a_get_value(&self) -> Result<ValueA, ClientError>;
}

#[async_trait]
pub(crate) trait ClientBTrait: Send + Sync {
    async fn b_get_value(&self) -> Result<ValueB, ClientError>;
}

pub(crate) struct ComponentA {
    b: Box<dyn ClientBTrait>,
}

impl ComponentA {
    pub fn new(b: Box<dyn ClientBTrait>) -> Self {
        Self { b }
    }

    pub async fn a_get_value(&self) -> ValueA {
        let b_value = self.b.b_get_value().await.unwrap();
        b_value.into()
    }
}

pub(crate) struct ComponentB {
    value: ValueB,
    _a: Box<dyn ClientATrait>,
}

impl ComponentB {
    pub fn new(value: ValueB, a: Box<dyn ClientATrait>) -> Self {
        Self { value, _a: a }
    }
    pub async fn b_get_value(&self) -> ValueB {
        self.value
    }
}
