use async_trait::async_trait;
use starknet_mempool_infra::component_client::ClientResult;

pub(crate) type ValueA = u32;
pub(crate) type ValueB = u8;

pub type ResultA = ClientResult<ValueA>;
pub type ResultB = ClientResult<ValueB>;

// TODO(Tsabary): add more messages / functions to the components.

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
        let b_value = self.b.b_get_value().await.unwrap();
        b_value.into()
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
