use async_trait::async_trait;
use tonic::transport::Channel;
use tonic::Request;
pub mod component_a_service {
    tonic::include_proto!("component_a_service");
}

use component_a_service::component_a_client::ComponentAClient;
// use component_a_service::component_a_server::{ComponentA, ComponentAServer};
use component_a_service::{ComponentAMessage, ComponentAResponse};

pub mod component_b_service {
    tonic::include_proto!("component_b_service");
}

use component_b_service::component_b_client::ComponentBClient;
// use component_b_service::component_b_server::{ComponentB, ComponentBServer};
use component_b_service::{ComponentBMessage, ComponentBResponse};

use crate::component_client_rpc::{ComponentClientRpc, Connection, RpcConnector};

type ValueA = i32;
type ValueB = u32;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BMessage {
    BGetValue,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BResponse {
    Value(ValueB),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AMessage {
    AGetValue,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AResponse {
    Value(ValueA),
}

struct AConnection {
    client: ComponentAClient<Channel>,
}

#[async_trait]
impl Connection<AMessage, AResponse> for AConnection {
    async fn send(&mut self, message: AMessage) -> AResponse {
        match message {
            AMessage::AGetValue => AResponse::Value(
                self.client
                    .get_value(Request::new(ComponentAMessage {}))
                    .await
                    .unwrap()
                    .get_ref()
                    .value,
            ),
        }
    }
}

struct BConnection {
    client: ComponentBClient<Channel>,
}

#[async_trait]
impl Connection<BMessage, BResponse> for BConnection {
    async fn send(&mut self, message: BMessage) -> BResponse {
        match message {
            BMessage::BGetValue => BResponse::Value(
                self.client
                    .get_value(Request::new(ComponentBMessage {}))
                    .await
                    .unwrap()
                    .get_ref()
                    .value,
            ),
        }
    }
}

struct TestConnector {
    ip: &'static str,
    port: u16,
}

impl TestConnector {
    pub fn new(ip: &'static str, port: u16) -> Self {
        Self { ip, port }
    }
}

#[async_trait]
impl RpcConnector<AMessage, AResponse> for TestConnector {
    async fn connect(
        &self,
    ) -> Result<Box<dyn Connection<AMessage, AResponse>>, tonic::transport::Error> {
        Ok(Box::new(AConnection {
            client: ComponentAClient::connect(format!("{}:{}", self.ip, self.port)).await?,
        }))
    }
}

#[async_trait]
impl RpcConnector<BMessage, BResponse> for TestConnector {
    async fn connect(
        &self,
    ) -> Result<Box<dyn Connection<BMessage, BResponse>>, tonic::transport::Error> {
        Ok(Box::new(BConnection {
            client: ComponentBClient::connect(format!("{}:{}", self.ip, self.port)).await?,
        }))
    }
}

#[tokio::test]
async fn test_setup() {
    let a_connector = TestConnector::new("[::1]", 10000);
    let b_connector = TestConnector::new("[::1]", 10001);

    let _a_client = ComponentClientRpc::<AMessage, AResponse>::new(&a_connector);
    let _b_client = ComponentClientRpc::<BMessage, BResponse>::new(&b_connector);
    // let component_a = ComponentA::new(Box::new(b_client));
    // let component_b = ComponentB::new(setup_value, Box::new(a_client));

    // let mut component_a_server = ComponentServer::new(component_a, rx_a);
    // let mut component_b_server = ComponentServer::new(component_b, rx_b);

    // task::spawn(async move {
    //     component_a_server.start().await;
    // });

    // task::spawn(async move {
    //     component_b_server.start().await;
    // });

    // verify_response(tx_a.clone(), expected_value).await;
}
