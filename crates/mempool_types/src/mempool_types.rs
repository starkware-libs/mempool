use mempool_infra::network_component::NetworkComponent;
use starknet_api::{
    core::{ContractAddress, Nonce},
    internal_transaction::InternalTransaction,
    transaction::{Tip, TransactionHash},
};

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub address: ContractAddress,
    pub state: AccountState,
}

#[derive(Debug)]
pub struct MempoolInput {
    pub tx: ThinTransaction,
    pub account: Account,
}

#[derive(Debug)]
pub enum Gateway2MempoolMessage {
    AddTx(InternalTransaction, AccountState),
}

pub type Mempool2GatewayMessage = ();

pub type GatewayNetworkComponent = NetworkComponent<Gateway2MempoolMessage, Mempool2GatewayMessage>;
pub type MempoolNetworkComponent = NetworkComponent<Mempool2GatewayMessage, Gateway2MempoolMessage>;
