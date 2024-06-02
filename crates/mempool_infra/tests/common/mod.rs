use async_trait::async_trait;
use starknet_mempool_infra::component_runner::{ComponentRunner, ComponentStartError};

pub(crate) type ValueA = u32;
pub(crate) type ValueB = u8;

// TODO(Tsabary): add more messages / functions to the components.

#[async_trait]
pub(crate) trait ComponentATrait: Send + Sync {
    async fn a_get_value(&self) -> ValueA;
}

#[async_trait]
pub(crate) trait ComponentBTrait: Send + Sync {
    async fn b_get_value(&self) -> ValueB;
}

pub(crate) struct ComponentA {
    b: Box<dyn ComponentBTrait>,
}

#[async_trait]
impl ComponentATrait for ComponentA {
    async fn a_get_value(&self) -> ValueA {
        let b_value = self.b.b_get_value().await;
        b_value.into()
    }
}

impl ComponentA {
    pub fn new(b: Box<dyn ComponentBTrait>) -> Self {
        Self { b }
    }
}

#[async_trait]
impl ComponentRunner for ComponentA {
    async fn start(&mut self) -> Result<(), ComponentStartError> {
        println!("ComponentA::start()");
        Ok(())
    }
}

pub(crate) struct ComponentB {
    value: ValueB,
    _a: Box<dyn ComponentATrait>,
}

#[async_trait]
impl ComponentBTrait for ComponentB {
    async fn b_get_value(&self) -> ValueB {
        self.value
    }
}

impl ComponentB {
    pub fn new(value: ValueB, a: Box<dyn ComponentATrait>) -> Self {
        Self { value, _a: a }
    }
}

#[async_trait]
impl ComponentRunner for ComponentB {
    async fn start(&mut self) -> Result<(), ComponentStartError> {
        println!("ComponentB::start()");
        Ok(())
    }
}
