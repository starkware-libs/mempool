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
pub enum GatewayMessage {
    None,                                // Input.
    AddTx(InternalTransaction, Account), // Output.
}

pub enum MempoolMessage {
    AddTx(InternalTransaction, Account), // Input.
    GetTxs(u8),                          // Output.
}

pub type GatewayNetworkComponent = NetworkComponent<GatewayMessage, MempoolMessage>;
pub type MempoolNetworkComponent = NetworkComponent<MempoolMessage, GatewayMessage>;

impl From<GatewayMessage> for MempoolMessage {
    fn from(item: GatewayMessage) -> Self {
        match item {
            GatewayMessage::AddTx(tx, state) => MempoolMessage::AddTx(tx, state),
            _ => unreachable!("Conversion not possible."),
        }
    }
}
