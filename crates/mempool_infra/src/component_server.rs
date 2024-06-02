use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

use crate::component_definitions::{ComponentRequestAndResponseSender, ComponentRequestHandler};
use crate::component_runner::ComponentRunner;

pub struct ComponentServer<Component, Request, Response>
where
    Component: ComponentRequestHandler<Request, Response> + ComponentRunner + Send + Sync,
    Request: Send + Sync,
    Response: Send + Sync,
{
    component: Component,
    rx: Receiver<ComponentRequestAndResponseSender<Request, Response>>,
    active_server: bool,
}

impl<Component, Request, Response> ComponentServer<Component, Request, Response>
where
    Component: ComponentRequestHandler<Request, Response> + ComponentRunner + Send + Sync,
    Request: Send + Sync,
    Response: Send + Sync,
{
    pub fn new(
        component: Component,
        rx: Receiver<ComponentRequestAndResponseSender<Request, Response>>,
    ) -> Self {
        Self { component, rx, active_server: true }
    }

    pub fn new_not_active(
        component: Component,
        rx: Receiver<ComponentRequestAndResponseSender<Request, Response>>,
    ) -> Self {
        Self { component, rx, active_server: false }
    }

    async fn request_response_loop(&mut self) {
        while let Some(request_and_res_tx) = self.rx.recv().await {
            let request = request_and_res_tx.request;
            let tx = request_and_res_tx.tx;

            let res = self.component.handle_request(request).await;

            tx.send(res).await.expect("Response connection should be open.");
        }
    }
}

#[async_trait]
pub trait CommunicationServer: Send + Sync {
    async fn start(&mut self);
}

#[async_trait]
impl<Component, Request, Response> CommunicationServer
    for ComponentServer<Component, Request, Response>
where
    Component: ComponentRequestHandler<Request, Response> + ComponentRunner + Send + Sync,
    Request: Send + Sync,
    Response: Send + Sync,
{
    async fn start(&mut self) {
        if self.active_server {
            self.request_response_loop().await;
        } else {
            match self.component.start().await {
                Ok(_) => println!("ComponentServer::start() completed."),
                Err(err) => println!("ComponentServer::start() failed: {:?}", err),
            }
        }
        println!("ComponentServer completed unexpectedly.");
    }

    // pub async fn start(&mut self) {
    //     tokio::select! {
    //         res = self.request_response_loop() => {
    //             println!("ComponentServer::request_response_loop() completed: {:?}", res);
    //         }
    //         res = self.component.start() => {
    //             match res {
    //                 Ok(_) => println!("ComponentServer::start() completed."),
    //                 Err(err) => println!("ComponentServer::start() failed: {:?}", err),
    //             }
    //         }
    //     }
    //     println!("ComponentServer completed unexpectedly.");
    // }
}
