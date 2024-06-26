use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

use crate::component_definitions::{ComponentRequestAndResponseSender, ComponentRequestHandler};
use crate::component_runner::ComponentRunner;

pub struct ComponentServer<Component, Request, Response>
where
    Component: ComponentRequestHandler<Request, Response>,
    Request: Send + Sync,
    Response: Send + Sync,
{
    component: Component,
    rx: Receiver<ComponentRequestAndResponseSender<Request, Response>>,
}

impl<Component, Request, Response> ComponentServer<Component, Request, Response>
where
    Component: ComponentRequestHandler<Request, Response>,
    Request: Send + Sync,
    Response: Send + Sync,
{
    pub fn new(
        component: Component,
        rx: Receiver<ComponentRequestAndResponseSender<Request, Response>>,
    ) -> Self {
        Self { component, rx }
    }

    pub async fn start_main_loop(&mut self) {
        while let Some(request_and_res_tx) = self.rx.recv().await {
            let request = request_and_res_tx.request;
            let tx = request_and_res_tx.tx;

            let res = self.component.handle_request(request).await;

            tx.send(res).await.expect("Response connection should be open.");
        }
    }
}

#[async_trait]
pub trait ComponentServerStarter: Send + Sync {
    async fn start(&mut self);
}

#[async_trait]
impl<Component, Request, Response> ComponentServerStarter
    for ComponentServer<Component, Request, Response>
where
    Component: ComponentRequestHandler<Request, Response> + Send + Sync,
    Request: Send + Sync,
    Response: Send + Sync,
{
    async fn start(&mut self) {
        self.start_main_loop().await;
        println!("ComponentServer completed unexpectedly.");
    }
}

pub struct EmptyServer<T: ComponentRunner + Send + Sync> {
    component: T,
}

impl<T: ComponentRunner + Send + Sync> EmptyServer<T> {
    pub fn new(component: T) -> Self {
        Self { component }
    }
}

#[async_trait]
impl<T: ComponentRunner + Send + Sync> ComponentServerStarter for EmptyServer<T> {
    async fn start(&mut self) {
        match self.component.start().await {
            Ok(_) => println!("ComponentServer::start() completed."),
            Err(err) => println!("ComponentServer::start() failed: {:?}", err),
        }
    }
}

pub fn create_empty_server<T: ComponentRunner + Send + Sync>(component: T) -> EmptyServer<T> {
    EmptyServer::new(component)
}
