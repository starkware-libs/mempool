use starknet_api::internal_transaction::InternalTransaction;

use mempool_infra::network_component::NetworkComponent;

#[derive(Debug)]
pub struct AccountState;



#[derive(Debug)]
pub enum GatewayIO {
    None, // Input.
    AddTx(InternalTransaction, AccountState), // Output.
}

pub enum MempoolIO {
    AddTx(InternalTransaction, AccountState), // Input.
    GetTxs(u8), // Output.
}

pub type GatewayNetworkComponent = NetworkComponent<GatewayIO, MempoolIO>;
pub type MempoolNetworkComponent = NetworkComponent<MempoolIO, GatewayIO>;
