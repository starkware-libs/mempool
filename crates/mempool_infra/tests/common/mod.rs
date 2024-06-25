use async_trait::async_trait;
use starknet_mempool_infra::component_client::ClientError;

pub(crate) type ValueA = u32;
pub(crate) type ValueB = u8;

// TODO(Tsabary): add more messages / functions to the components.

pub type ClientResult<T> = Result<T, ClientError>;
pub type AClientResult = ClientResult<ValueA>;
pub type BClientResult = ClientResult<ValueB>;

#[async_trait]
pub(crate) trait AClient: Send + Sync {
    async fn a_get_value(&self) -> AClientResult;
}

#[async_trait]
pub(crate) trait BClient: Send + Sync {
    async fn b_get_value(&self) -> BClientResult;
}

pub(crate) struct ComponentA {
    b: Box<dyn BClient>,
}

impl ComponentA {
    pub fn new(b: Box<dyn BClient>) -> Self {
        Self { b }
    }

    pub async fn a_get_value(&self) -> ValueA {
        let b_value = self.b.b_get_value().await.unwrap();
        b_value.into()
    }
}

pub(crate) struct ComponentB {
    value: ValueB,
    _a: Box<dyn AClient>,
}

impl ComponentB {
    pub fn new(value: ValueB, a: Box<dyn AClient>) -> Self {
        Self { value, _a: a }
    }
    pub fn b_get_value(&self) -> ValueB {
        self.value
    }
}
