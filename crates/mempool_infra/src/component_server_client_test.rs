use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task;

use crate::component_client::ComponentClient;

type ValueA = u32;
type ValueB = u8;

use crate::component_server::{
    ComponentMessageExecutor, ComponentServer, MessageAndResponseSender,
};

#[async_trait]
trait ComponentATrait: Send + Sync {
    async fn func_a(&self) -> ValueA;
}

#[async_trait]
trait ComponentBTrait: Send + Sync {
    async fn func_b(&self) -> ValueB;
}

struct ComponentA {
    b: Box<dyn ComponentBTrait>,
}

#[async_trait]
impl ComponentATrait for ComponentA {
    async fn func_a(&self) -> ValueA {
        let b_value = self.b.func_b().await;
        b_value.into()
    }
}

impl ComponentA {
    fn new(b: Box<dyn ComponentBTrait>) -> Self {
        Self { b }
    }
}

// todo find which of these derives is needed
// todo add more messages
// todo send messages from b to a

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComponentAMessages {
    GetValue,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComponentAResponses {
    Value(ValueA),
}

#[async_trait]
impl ComponentATrait for ComponentClient<ComponentAMessages, ComponentAResponses> {
    async fn func_a(&self) -> ValueA {
        let res = self.send(ComponentAMessages::GetValue).await;
        match res {
            ComponentAResponses::Value(value) => value,
        }
    }
}

#[async_trait]
impl ComponentMessageExecutor<ComponentAMessages, ComponentAResponses> for ComponentA {
    async fn execute(&self, message: ComponentAMessages) -> ComponentAResponses {
        match message {
            ComponentAMessages::GetValue => ComponentAResponses::Value(self.func_a().await),
        }
    }
}

struct ComponentB {
    value: ValueB,
    _a: Box<dyn ComponentATrait>,
}

#[async_trait]
impl ComponentBTrait for ComponentB {
    async fn func_b(&self) -> ValueB {
        self.value
    }
}

impl ComponentB {
    fn new(value: ValueB, a: Box<dyn ComponentATrait>) -> Self {
        Self { value, _a: a }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComponentBMessages {
    GetShmalue,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComponentBResponses {
    Shmalue(ValueB),
}

#[async_trait]
impl ComponentBTrait for ComponentClient<ComponentBMessages, ComponentBResponses> {
    async fn func_b(&self) -> ValueB {
        let res = self.send(ComponentBMessages::GetShmalue).await;
        match res {
            ComponentBResponses::Shmalue(value) => value,
        }
    }
}

#[async_trait]
impl ComponentMessageExecutor<ComponentBMessages, ComponentBResponses> for ComponentB {
    async fn execute(&self, message: ComponentBMessages) -> ComponentBResponses {
        match message {
            ComponentBMessages::GetShmalue => ComponentBResponses::Shmalue(self.func_b().await),
        }
    }
}

async fn verify_response(
    tx_a: Sender<MessageAndResponseSender<ComponentAMessages, ComponentAResponses>>,
) {
    let (tx_a_main, mut rx_a_main) = channel::<ComponentAResponses>(1);

    let message_and_res_tx: MessageAndResponseSender<ComponentAMessages, ComponentAResponses> =
        MessageAndResponseSender { message: ComponentAMessages::GetValue, tx: tx_a_main };

    tx_a.send(message_and_res_tx).await.unwrap();

    let res = rx_a_main.recv().await.unwrap();
    match res {
        ComponentAResponses::Value(value) => {
            assert_eq!(value, 30);
        }
    }
}

#[tokio::test]
async fn test_setup() {
    let (tx_a, rx_a) =
        channel::<MessageAndResponseSender<ComponentAMessages, ComponentAResponses>>(32);
    let (tx_b, rx_b) =
        channel::<MessageAndResponseSender<ComponentBMessages, ComponentBResponses>>(32);

    let a_client = ComponentClient::new(tx_a.clone());
    let b_client = ComponentClient::new(tx_b.clone());

    let component_a = ComponentA::new(Box::new(b_client));
    let component_b = ComponentB::new(30, Box::new(a_client));

    let mut component_a_server = ComponentServer::new(component_a, rx_a);
    let mut component_b_server = ComponentServer::new(component_b, rx_b);

    task::spawn(async move {
        component_a_server.start().await;
    });

    task::spawn(async move {
        component_b_server.start().await;
    });

    verify_response(tx_a.clone()).await;
}
