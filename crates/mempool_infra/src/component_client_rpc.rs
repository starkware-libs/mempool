use async_trait::async_trait;
use tonic::transport::Error;

#[cfg(test)]
#[path = "component_server_client_rpc_test.rs"]
mod component_server_client_rpc_test;

#[async_trait]
pub trait Connection<M, R> {
    async fn send(&mut self, message: M) -> R;
}

#[async_trait]
pub trait RpcConnector<M, R> {
    async fn connect(&self) -> Result<Box<dyn Connection<M, R>>, Error>;
}

#[derive(Clone)]
pub struct ComponentClientRpc<'a, M, R> {
    rpc_connector: &'a dyn RpcConnector<M, R>,
}

impl<'a, M, R> ComponentClientRpc<'a, M, R>
where
    M: Send + Sync,
    R: Send + Sync,
{
    pub fn new(rpc_connector: &'a impl RpcConnector<M, R>) -> Self {
        Self { rpc_connector }
    }

    pub async fn send(&self, message: M) -> R {
        self.rpc_connector.connect().await.unwrap().send(message).await
    }
}
