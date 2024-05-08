use std::marker::PhantomData;

use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;
use tonic::transport::{Channel, Server};
use tonic::{Request, Response, Status};

pub mod rpcnetworkcomponentsender {
    tonic::include_proto!("rpcnetworkcomponentsender");
}

use rpcnetworkcomponentsender::rpc_network_component_sender_client::RpcNetworkComponentSenderClient;
use rpcnetworkcomponentsender::rpc_network_component_sender_server::{
    RpcNetworkComponentSender, RpcNetworkComponentSenderServer,
};
use rpcnetworkcomponentsender::RpcNetworkComponentMessage;

use crate::network_component::{CommunicationInterface, NetworkComponentError};

type Port = u16;

impl From<i32> for RpcNetworkComponentMessage {
    fn from(data: i32) -> Self {
        RpcNetworkComponentMessage { data_i32: data, data_u32: 0 }
    }
}

impl From<RpcNetworkComponentMessage> for i32 {
    fn from(message: RpcNetworkComponentMessage) -> Self {
        message.data_i32
    }
}

impl From<u32> for RpcNetworkComponentMessage {
    fn from(data: u32) -> Self {
        RpcNetworkComponentMessage { data_u32: data, data_i32: 0 }
    }
}

impl From<RpcNetworkComponentMessage> for u32 {
    fn from(message: RpcNetworkComponentMessage) -> Self {
        message.data_u32
    }
}

pub struct RpcNetworkComponentSenderService<T> {
    tx: Sender<T>,
}

#[tonic::async_trait]
impl<T> RpcNetworkComponentSender for RpcNetworkComponentSenderService<T>
where
    T: Send + Sync + 'static + From<RpcNetworkComponentMessage>,
{
    async fn send_message(
        &self,
        request: Request<RpcNetworkComponentMessage>,
    ) -> Result<Response<()>, Status> {
        let message: RpcNetworkComponentMessage = request.into_inner();
        self.tx.send(message.into()).await.map_err(|e| Status::from_error(Box::new(e)))?;
        Ok(Response::new(()))
    }
}

pub struct RpcNetworkComponent<S, R> {
    rx: Receiver<R>,
    client: Option<RpcNetworkComponentSenderClient<Channel>>,
    send_port: Port,
    sender_type: PhantomData<S>,
}

impl<S, R> RpcNetworkComponent<S, R>
where
    S: Send + Sync,
    R: Send + Sync,
    RpcNetworkComponentSenderService<R>: RpcNetworkComponentSender,
{
    pub fn new(send_port: Port, recv_port: Port) -> Self {
        let (tx_service, rx) = channel::<R>(1);

        task::spawn(async move {
            let addr = format!("[::1]:{recv_port}").parse().expect("Parsing should succeed");
            let sender_service = RpcNetworkComponentSenderService::<R> { tx: tx_service };
            let server = RpcNetworkComponentSenderServer::new(sender_service);
            if let Err(e) = Server::builder().add_service(server).serve(addr).await {
                println!("{e}");
            }
        });

        Self { rx, client: None, send_port, sender_type: Default::default() }
    }
}

#[async_trait]
impl<S, R> CommunicationInterface for RpcNetworkComponent<S, R>
where
    S: Send + Sync,
    R: Send + Sync,
    RpcNetworkComponentMessage: From<S>,
{
    type SendType = S;
    type ReceiveType = R;

    async fn send(&mut self, message: Self::SendType) -> Result<(), NetworkComponentError> {
        if self.client.is_none() {
            let client = loop {
                let addr: String = format!("http://[::1]:{}", self.send_port);
                if let Ok(client) = RpcNetworkComponentSenderClient::connect(addr).await {
                    break client;
                }
            };

            self.client = Some(client);
        }

        if let Some(client) = self.client.as_mut() {
            if client.send_message(Request::new(message.into())).await.is_ok() {
                return Ok(());
            }
        }

        return Err(NetworkComponentError::SendFailure);
    }

    async fn recv(&mut self) -> Option<Self::ReceiveType> {
        self.rx.recv().await
    }
}
