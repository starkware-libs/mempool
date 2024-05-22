use async_trait::async_trait;
use mempool_infra::component_client::ComponentClient;
use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::{Tip, TransactionHash};

use crate::errors::MempoolError;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ThinTransaction {
    pub contract_address: ContractAddress,
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

pub type MempoolResult<T> = Result<T, MempoolError>;
#[async_trait]
pub trait MempoolTrait: Send + Sync {
    async fn async_add_tx(&mut self, tx: ThinTransaction, account: Account) -> MempoolResult<()>;
    async fn async_get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>>;
}

#[derive(Debug)]
pub enum MempoolMessages {
    AsyncAddTransaction(ThinTransaction, Account),
    AsyncGetTxs(usize),
}

#[derive(Debug)]
pub enum MempoolResponses {
    AsyncAddTransaction(MempoolResult<()>),
    AsyncGetTxs(MempoolResult<Vec<ThinTransaction>>),
}

#[async_trait]
impl MempoolTrait for ComponentClient<MempoolMessages, MempoolResponses> {
    async fn async_add_tx(&mut self, tx: ThinTransaction, account: Account) -> MempoolResult<()> {
        let res = self.send(MempoolMessages::AsyncAddTransaction(tx, account)).await;
        match res {
            MempoolResponses::AsyncAddTransaction(res) => res,
            _ => panic!("Unexpected response type."),
        }
    }

    async fn async_get_txs(&mut self, n_txs: usize) -> MempoolResult<Vec<ThinTransaction>> {
        let res = self.send(MempoolMessages::AsyncGetTxs(n_txs)).await;
        match res {
            MempoolResponses::AsyncGetTxs(res) => res,
            _ => panic!("Unexpected response type."),
        }
    }
}
