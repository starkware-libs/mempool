mod component_a_service {
    tonic::include_proto!("component_a_service");
}
mod component_b_service {
    tonic::include_proto!("component_b_service");
}
mod common;

use std::net::IpAddr;

use async_trait::async_trait;
use common::{AClientTrait, BClientTrait};
use component_a_service::remote_a_client::RemoteAClient;
use component_a_service::remote_a_server::{RemoteA, RemoteAServer};
use component_a_service::{AGetValueMessage, AGetValueReturnMessage};
use component_b_service::remote_b_client::RemoteBClient;
use component_b_service::remote_b_server::{RemoteB, RemoteBServer};
use component_b_service::{BGetValueMessage, BGetValueReturnMessage};
use starknet_mempool_infra::component_client::ClientError;
use starknet_mempool_infra::component_client_rpc::ComponentClientRpc;
use starknet_mempool_infra::component_server_rpc::ComponentServerRpc;
use tokio::task;
use tonic::transport::Server;
use tonic::{Response, Status};

use crate::common::{ClientResult, ComponentA, ComponentB, ValueA, ValueB};

#[async_trait]
impl AClientTrait for ComponentClientRpc<ComponentA> {
    async fn a_get_value(&self) -> ClientResult<ValueA> {
        let mut client = match RemoteAClient::connect(self.dst.clone()).await {
            Ok(client) => client,
            Err(e) => return Err(ClientError::ConnectionFailure(e)),
        };

        let response = match client.remote_a_get_value(AGetValueMessage {}).await {
            Ok(response) => response,
            Err(e) => return Err(ClientError::ResponseFailure(e)),
        };

        Ok(response.into_inner().value)
    }
}

#[async_trait]
impl BClientTrait for ComponentClientRpc<ComponentB> {
    async fn b_get_value(&self) -> ClientResult<ValueB> {
        let mut client = match RemoteBClient::connect(self.dst.clone()).await {
            Ok(client) => client,
            Err(e) => return Err(ClientError::ConnectionFailure(e)),
        };

        let response = match client.remote_b_get_value(BGetValueMessage {}).await {
            Ok(response) => response,
            Err(e) => return Err(ClientError::ResponseFailure(e)),
        };

        Ok(response.into_inner().value.try_into().unwrap())
    }
}

#[async_trait]
impl RemoteA for ComponentA {
    async fn remote_a_get_value(
        &self,
        _request: tonic::Request<AGetValueMessage>,
    ) -> Result<Response<AGetValueReturnMessage>, Status> {
        Ok(Response::new(AGetValueReturnMessage { value: self.a_get_value().await }))
    }
}

#[async_trait]
pub trait ServerStart {
    async fn start(&mut self);
}

#[async_trait]
impl ServerStart for ComponentServerRpc<ComponentA> {
    async fn start(&mut self) {
        let svc = RemoteAServer::new(self.component.take().unwrap());
        Server::builder().add_service(svc).serve(self.address).await.unwrap();
    }
}

#[async_trait]
impl RemoteB for ComponentB {
    async fn remote_b_get_value(
        &self,
        _request: tonic::Request<BGetValueMessage>,
    ) -> Result<Response<BGetValueReturnMessage>, Status> {
        Ok(Response::new(BGetValueReturnMessage { value: self.b_get_value().await.into() }))
    }
}

#[async_trait]
impl ServerStart for ComponentServerRpc<ComponentB> {
    async fn start(&mut self) {
        let svc = RemoteBServer::new(self.component.take().unwrap());
        Server::builder().add_service(svc).serve(self.address).await.unwrap();
    }
}

async fn verify_response(ip_address: IpAddr, port: u16, expected_value: ValueA) {
    let a_client = ComponentClientRpc::<ComponentA>::new(ip_address, port);

    let returned_value = a_client.a_get_value().await.expect("Value should be returned");
    assert_eq!(returned_value, expected_value);
}

#[tokio::test]
async fn test_setup() {
    let setup_value: ValueB = 60;
    let expected_value: ValueA = setup_value.into();

    let local_ip = "::1".parse().unwrap();
    let a_port = 10000;
    let b_port = 10001;

    let a_client = ComponentClientRpc::<ComponentA>::new(local_ip, a_port);
    let b_client = ComponentClientRpc::<ComponentB>::new(local_ip, b_port);

    let component_a = ComponentA::new(Box::new(b_client));
    let component_b = ComponentB::new(setup_value, Box::new(a_client));

    let mut component_a_server =
        ComponentServerRpc::<ComponentA>::new(component_a, local_ip, a_port);
    let mut component_b_server =
        ComponentServerRpc::<ComponentB>::new(component_b, local_ip, b_port);

    task::spawn(async move {
        component_a_server.start().await;
    });

    task::spawn(async move {
        component_b_server.start().await;
    });

    task::yield_now().await;

    verify_response(local_ip, a_port, expected_value).await;
}
