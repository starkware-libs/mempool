use std::marker::PhantomData;

use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;
use tonic::transport::{Channel, Server};
use tonic::{Request, Response, Status};

pub mod mysender {
    tonic::include_proto!("mysender");
}

use mysender::my_sender_client::MySenderClient;
use mysender::my_sender_server::{MySender, MySenderServer};
use mysender::MyMessage;

use crate::network_component::{CommunicationInterface, NetworkComponentError};

type Port = u16;

impl From<i32> for MyMessage {
    fn from(data: i32) -> Self {
        MyMessage { data_i32: data, data_u32: 0 }
    }
}

impl From<MyMessage> for i32 {
    fn from(message: MyMessage) -> Self {
        message.data_i32
    }
}

impl From<u32> for MyMessage {
    fn from(data: u32) -> Self {
        MyMessage { data_u32: data, data_i32: 0 }
    }
}

impl From<MyMessage> for u32 {
    fn from(message: MyMessage) -> Self {
        message.data_u32
    }
}

pub struct MySenderService<T> {
    tx: Sender<T>,
}

#[tonic::async_trait]
impl<T> MySender for MySenderService<T>
where
    T: Send + Sync + 'static + From<MyMessage>,
{
    async fn send_message(&self, request: Request<MyMessage>) -> Result<Response<()>, Status> {
        let message: MyMessage = request.into_inner();
        self.tx.send(message.into()).await.map_err(|e| Status::from_error(Box::new(e)))?;
        Ok(Response::new(()))
    }
}

pub struct TonicNetworkComponent<S, R> {
    rx: Receiver<R>,
    client: Option<MySenderClient<Channel>>,
    send_port: Port,
    sender_type: PhantomData<S>,
}

impl<S, R> TonicNetworkComponent<S, R>
where
    S: Send + Sync,
    R: Send + Sync,
    MySenderService<R>: MySender,
{
    pub fn new(send_port: Port, recv_port: Port) -> Self {
        let (tx_service, rx) = channel::<R>(1);

        task::spawn(async move {
            let addr = format!("[::1]:{recv_port}").parse().expect("Parsing should succeed");
            let sender_service = MySenderService::<R> { tx: tx_service };
            let server = MySenderServer::new(sender_service);
            if let Err(e) = Server::builder().add_service(server).serve(addr).await {
                println!("{e}");
            }
        });

        Self { rx, client: None, send_port, sender_type: Default::default() }
    }
}

#[async_trait]
impl<S, R> CommunicationInterface for TonicNetworkComponent<S, R>
where
    S: Send + Sync,
    R: Send + Sync,
    MyMessage: From<S>,
{
    type SendType = S;
    type ReceiveType = R;

    async fn send(&mut self, message: Self::SendType) -> Result<(), NetworkComponentError> {
        if self.client.is_none() {
            let client = loop {
                let addr: String = format!("http://[::1]:{}", self.send_port);
                if let Ok(client) = MySenderClient::connect(addr).await {
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
