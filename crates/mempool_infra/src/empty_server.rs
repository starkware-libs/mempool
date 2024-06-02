use async_trait::async_trait;
use tokio::sync::mpsc::channel;

use crate::component_definitions::{ComponentRequestAndResponseSender, ComponentRequestHandler};
use crate::component_runner::{ComponentRunner, ComponentStartError};
use crate::component_server::ComponentServer;

pub struct EmptyCommunicationWrapper<T: ComponentRunner + Send> {
    component: T,
}

impl<T: ComponentRunner + Send> EmptyCommunicationWrapper<T> {
    pub fn new(component: T) -> Self {
        Self { component }
    }
}

#[async_trait]
impl<T: ComponentRunner + Send> ComponentRunner for EmptyCommunicationWrapper<T> {
    async fn start(&mut self) -> Result<(), ComponentStartError> {
        self.component.start().await
    }
}

pub struct EmptyRequest {}
pub struct EmptyResponse {}

#[async_trait]
impl<T: ComponentRunner + Send> ComponentRequestHandler<EmptyRequest, EmptyResponse>
    for EmptyCommunicationWrapper<T>
{
    async fn handle_request(&mut self, _request: EmptyRequest) -> EmptyResponse {
        EmptyResponse {}
    }
}

pub type EmptyServer<T> =
    ComponentServer<EmptyCommunicationWrapper<T>, EmptyRequest, EmptyResponse>;
pub type EmptyRequestAndResponseSender =
    ComponentRequestAndResponseSender<EmptyRequest, EmptyResponse>;

pub fn create_empty_server<T: ComponentRunner + Send + Sync>(component: T) -> EmptyServer<T> {
    let (_, rx) = channel::<EmptyRequestAndResponseSender>(1);

    let empty_communication_wrapper = EmptyCommunicationWrapper::new(component);
    EmptyServer::new_not_active(empty_communication_wrapper, rx)
}
