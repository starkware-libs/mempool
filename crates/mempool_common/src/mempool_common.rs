use starknet_api::internal_transaction::InternalTransaction;

use mempool_infra::network_component::NetworkComponent;

#[derive(Debug)]
pub struct AccountState;

#[derive(Debug)]
pub enum GatewayMessage {
    None,                                     // Input.
    AddTx(InternalTransaction, AccountState), // Output.
}

pub enum MempoolMessage {
    AddTx(InternalTransaction, AccountState), // Input.
    GetTxs(u8),                               // Output.
}

pub type GatewayNetworkComponent = NetworkComponent<GatewayMessage, MempoolMessage>;
pub type MempoolNetworkComponent = NetworkComponent<MempoolMessage, GatewayMessage>;
