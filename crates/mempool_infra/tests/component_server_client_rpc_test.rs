mod component_a_service {
    tonic::include_proto!("component_a_service");
}
mod component_b_service {
    tonic::include_proto!("component_b_service");
}

mod common;

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use async_trait::async_trait;
use common::{ComponentATrait, ComponentBTrait};
use component_a_service::remote_a_client::RemoteAClient;
use component_a_service::remote_a_server::{RemoteA, RemoteAServer};
use component_a_service::{AGetValueMessage, AGetValueReturnMessage};
use component_b_service::remote_b_client::RemoteBClient;
use component_b_service::remote_b_server::{RemoteB, RemoteBServer};
use component_b_service::{BGetValueMessage, BGetValueReturnMessage};
use tokio::sync::Mutex;
use tokio::task;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::common::{ComponentA, ComponentB, ValueA, ValueB};

fn construct_url(ip_address: IpAddr, port: u16) -> String {
    match ip_address {
        IpAddr::V4(ip_address) => format!("http://{}:{}/", ip_address, port),
        IpAddr::V6(ip_address) => format!("http://[{}]:{}/", ip_address, port),
    }
}

struct ComponentAClientRpc {
    dst: String,
}

impl ComponentAClientRpc {
    fn new(ip_address: IpAddr, port: u16) -> Self {
        Self { dst: construct_url(ip_address, port) }
    }
}

#[async_trait]
impl ComponentATrait for ComponentAClientRpc {
    async fn a_get_value(&self) -> ValueA {
        let Ok(mut client) = RemoteAClient::connect(self.dst.clone()).await else {
            panic!("Could not connect to server");
        };

        let Ok(response) = client.a_get_value(Request::new(AGetValueMessage {})).await else {
            panic!("Could not get response from server");
        };

        response.get_ref().value
    }
}

struct ComponentBClientRpc {
    dst: String,
}

impl ComponentBClientRpc {
    fn new(ip_address: IpAddr, port: u16) -> Self {
        Self { dst: construct_url(ip_address, port) }
    }
}

#[async_trait]
impl ComponentBTrait for ComponentBClientRpc {
    async fn b_get_value(&self) -> ValueB {
        let Ok(mut client) = RemoteBClient::connect(self.dst.clone()).await else {
            panic!("Could not connect to server");
        };

        let Ok(response) = client.b_get_value(Request::new(BGetValueMessage {})).await else {
            panic!("Could not get response from server");
        };

        response.get_ref().value.try_into().unwrap()
    }
}

#[async_trait]
impl RemoteA for Arc<Mutex<ComponentA>> {
    async fn a_get_value(
        &self,
        _request: tonic::Request<AGetValueMessage>,
    ) -> Result<Response<AGetValueReturnMessage>, Status> {
        let a = self.lock().await;
        Ok(Response::new(AGetValueReturnMessage { value: a.a_get_value().await }))
    }
}

struct ComponentAServerRpc {
    a: Arc<Mutex<ComponentA>>,
    address: SocketAddr,
}

impl ComponentAServerRpc {
    fn new(a: ComponentA, ip_address: IpAddr, port: u16) -> Self {
        Self { a: Arc::new(Mutex::new(a)), address: SocketAddr::new(ip_address, port) }
    }

    async fn start(&self) {
        let svc = RemoteAServer::new(Arc::clone(&self.a));
        Server::builder().add_service(svc).serve(self.address).await.unwrap();
    }
}

#[async_trait]
impl RemoteB for Arc<Mutex<ComponentB>> {
    async fn b_get_value(
        &self,
        _request: tonic::Request<BGetValueMessage>,
    ) -> Result<Response<BGetValueReturnMessage>, Status> {
        let b = self.lock().await;
        Ok(Response::new(BGetValueReturnMessage { value: b.b_get_value().await.into() }))
    }
}

struct ComponentBServerRpc {
    b: Arc<Mutex<ComponentB>>,
    address: SocketAddr,
}

impl ComponentBServerRpc {
    fn new(b: ComponentB, ip_address: IpAddr, port: u16) -> Self {
        Self { b: Arc::new(Mutex::new(b)), address: SocketAddr::new(ip_address, port) }
    }

    async fn start(&self) {
        let svc = RemoteBServer::new(Arc::clone(&self.b));
        Server::builder().add_service(svc).serve(self.address).await.unwrap();
    }
}

async fn verify_response(ip_address: IpAddr, port: u16, expected_value: ValueA) {
    let Ok(mut client) = RemoteAClient::connect(construct_url(ip_address, port)).await else {
        panic!("Verify failed: Could not connect to server");
    };

    let Ok(response) = client.a_get_value(Request::new(AGetValueMessage {})).await else {
        panic!("Verify failed: Could not get response from server");
    };

    let value = response.get_ref().value;
    assert_eq!(value, expected_value);
}

#[tokio::test]
async fn test_setup() {
    let setup_value: ValueB = 60;
    let expected_value: ValueA = setup_value.into();

    let local_ip = "::1".parse().unwrap();
    let a_port = 10000;
    let b_port = 10001;

    let a_client = ComponentAClientRpc::new(local_ip, a_port);
    let b_client = ComponentBClientRpc::new(local_ip, b_port);

    let component_a = ComponentA::new(Box::new(b_client));
    let component_b = ComponentB::new(setup_value, Box::new(a_client));

    let component_a_server = ComponentAServerRpc::new(component_a, local_ip, a_port);
    let component_b_server = ComponentBServerRpc::new(component_b, local_ip, b_port);

    task::spawn(async move {
        component_a_server.start().await;
    });

    task::spawn(async move {
        component_b_server.start().await;
    });

    task::yield_now().await;

    verify_response(local_ip, a_port, expected_value).await;
}
