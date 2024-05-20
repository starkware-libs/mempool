use async_trait::async_trait;
use mempool_infra::component_client::ComponentClient;
use mempool_infra::component_server::ComponentRequestAndResponseSender;
use mempool_infra::network_component::NetworkComponent;
use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::{Tip, TransactionHash};
use thiserror::Error;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::errors::MempoolError;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ThinTransaction {
    pub sender_address: ContractAddress,
    pub tx_hash: TransactionHash,
    pub tip: Tip,
    pub nonce: Nonce,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AccountState {
    pub nonce: Nonce,
    // TODO: add balance field when needed.
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Account {
    // TODO(Ayelet): Consider removing this field as it is duplicated in ThinTransaction.
    pub address: ContractAddress,
    pub state: AccountState,
}

#[derive(Debug, Default)]
pub struct MempoolInput {
    pub tx: ThinTransaction,
    pub account: Account,
}

#[derive(Debug)]
pub enum GatewayToMempoolMessage {
    AddTransaction(MempoolInput),
}

// TODO: Consider using `NetworkComponent` instead of defining the channels here.
// Currently, facing technical issues when using `NetworkComponent`.
pub struct BatcherToMempoolChannels {
    pub rx: Receiver<BatcherToMempoolMessage>,
    pub tx: Sender<MempoolToBatcherMessage>,
}

pub enum BatcherToMempoolMessage {
    GetTransactions(usize),
}
pub type MempoolToGatewayMessage = ();
pub type MempoolToBatcherMessage = Vec<ThinTransaction>;

pub type BatcherMempoolNetworkComponent =
    NetworkComponent<BatcherToMempoolMessage, MempoolToBatcherMessage>;
pub type MempoolBatcherNetworkComponent =
    NetworkComponent<MempoolToBatcherMessage, BatcherToMempoolMessage>;

pub type GatewayNetworkComponent =
    NetworkComponent<GatewayToMempoolMessage, MempoolToGatewayMessage>;
pub type MempoolNetworkComponent =
    NetworkComponent<MempoolToGatewayMessage, GatewayToMempoolMessage>;

pub type MempoolResult<T> = Result<T, MempoolError>;

// TODO(Tsabary, 1/6/2024): Move communication-related definitions to a separate file.
#[derive(Debug, Error)]
pub enum NetworkError {}

#[async_trait]
pub trait MempoolClientInterface: Send + Sync {
    async fn add_tx(&self, mempool_input: MempoolInput) -> Result<MempoolResult<()>, NetworkError>;
    async fn get_txs(
        &self,
        n_txs: usize,
    ) -> Result<MempoolResult<Vec<ThinTransaction>>, NetworkError>;
}

#[derive(Debug)]
pub enum MempoolClientRequest {
    AddTransaction(MempoolInput),
    GetTransactions(usize),
}

#[derive(Debug)]
pub enum MempoolClientResponse {
    AddTransaction(MempoolResult<()>),
    GetTransactions(MempoolResult<Vec<ThinTransaction>>),
}

pub type MempoolClient = ComponentClient<MempoolClientRequest, MempoolClientResponse>;
pub type MempoolMessageAndResponseSender =
    ComponentRequestAndResponseSender<MempoolClientRequest, MempoolClientResponse>;

#[async_trait]
impl MempoolClientInterface for MempoolClient {
    async fn add_tx(&self, mempool_input: MempoolInput) -> Result<MempoolResult<()>, NetworkError> {
        let add_tx_message = MempoolClientRequest::AddTransaction(mempool_input);
        let res = self.send(add_tx_message).await;
        match res {
            MempoolClientResponse::AddTransaction(res) => Result::Ok(res),
            _ => panic!("Unexpected response type."),
        }
    }

    async fn get_txs(
        &self,
        n_txs: usize,
    ) -> Result<MempoolResult<Vec<ThinTransaction>>, NetworkError> {
        let get_txs_message = MempoolClientRequest::GetTransactions(n_txs);
        let res = self.send(get_txs_message).await;
        match res {
            MempoolClientResponse::GetTransactions(res) => Result::Ok(res),
            _ => panic!("Unexpected response type."),
        }
    }
}
