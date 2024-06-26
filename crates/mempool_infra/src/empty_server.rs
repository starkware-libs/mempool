use async_trait::async_trait;

use crate::component_runner::ComponentRunner;
use crate::component_server::CommunicationServer;

pub struct EmptyServer<T: ComponentRunner + Send> {
    component: T,
}

impl<T: ComponentRunner + Send + Sync> EmptyServer<T> {
    pub fn new(component: T) -> Self {
        Self { component }
    }
}

#[async_trait]
impl<T: ComponentRunner + Send + Sync> CommunicationServer for EmptyServer<T> {
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
