use std::sync::Arc;

use async_trait::async_trait;
use starknet_mempool_infra::component_client::{ClientError, ClientResult, ComponentClient};
use starknet_mempool_infra::component_definitions::ComponentRequestAndResponseSender;

use crate::errors::MempoolError;
use crate::mempool_types::{MempoolInput, ThinTransaction};

pub type MempoolClientImpl = ComponentClient<MempoolRequest, MempoolResponse>;
pub type MempoolResult<T> = Result<T, MempoolError>;
pub type MempoolRequestAndResponseSender =
    ComponentRequestAndResponseSender<MempoolRequest, MempoolResponse>;
pub type SharedMempoolClient = Arc<dyn MempoolClient>;

/// Serves as the mempool's shared interface. Requires `Send + Sync` to allow transferring and
/// sharing resources (inputs, futures) across threads.
#[async_trait]
pub trait MempoolClient: Send + Sync {
    async fn add_tx(&self, mempool_input: MempoolInput) -> ClientResult<MempoolResult<()>>;
    async fn get_txs(&self, n_txs: usize) -> ClientResult<MempoolResult<Vec<ThinTransaction>>>;
}

#[derive(Debug)]
pub enum MempoolRequest {
    AddTransaction(MempoolInput),
    GetTransactions(usize),
}

#[derive(Debug)]
pub enum MempoolResponse {
    AddTransaction(MempoolResult<()>),
    GetTransactions(MempoolResult<Vec<ThinTransaction>>),
}

#[async_trait]
impl MempoolClient for MempoolClientImpl {
    async fn add_tx(&self, mempool_input: MempoolInput) -> ClientResult<MempoolResult<()>> {
        let request = MempoolRequest::AddTransaction(mempool_input);
        let response = self.send(request).await?;
        match response {
            MempoolResponse::AddTransaction(response) => Ok(response),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    async fn get_txs(&self, n_txs: usize) -> ClientResult<MempoolResult<Vec<ThinTransaction>>> {
        let request = MempoolRequest::GetTransactions(n_txs);
        let response = self.send(request).await?;
        match response {
            MempoolResponse::GetTransactions(response) => Ok(response),
            _ => Err(ClientError::UnexpectedResponse),
        }
    }
}
