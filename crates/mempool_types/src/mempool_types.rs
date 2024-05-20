use async_trait::async_trait;
use mempool_infra::component_client::ComponentClient;
use mempool_infra::network_component::NetworkComponent;
use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::{Tip, TransactionHash};

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

#[derive(Debug)]
pub enum GatewayToMempoolMessage {
    AddTransaction(MempoolInput),
}

pub type MempoolToGatewayMessage = ();

pub type GatewayNetworkComponent =
    NetworkComponent<GatewayToMempoolMessage, MempoolToGatewayMessage>;
pub type MempoolNetworkComponent =
    NetworkComponent<MempoolToGatewayMessage, GatewayToMempoolMessage>;

#[async_trait]
pub trait MempoolTrait: Send + Sync {
    async fn async_add_tx(&mut self, tx: ThinTransaction, account: Account) -> ();
}

#[derive(Clone, Debug)]
pub enum MempoolMessages {
    AsyncAddTransaction(ThinTransaction, Account),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MempoolResponses {
    AsyncAddTransaction(),
}

#[async_trait]
impl MempoolTrait for ComponentClient<MempoolMessages, MempoolResponses> {
    async fn async_add_tx(&mut self, tx: ThinTransaction, account: Account) {
        self.send(MempoolMessages::AsyncAddTransaction(tx, account)).await;
    }
}
