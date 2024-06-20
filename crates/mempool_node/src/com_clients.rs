use std::sync::Arc;

use starknet_mempool_types::communication::{MempoolClientImpl, SharedMempoolClient};

use crate::communication::MempoolNodeCommunication;
use crate::config::MempoolNodeConfig;

pub struct CommClients {
    pub mempool_client: Option<SharedMempoolClient>,
}

pub fn create_node_clients(
    config: &MempoolNodeConfig,
    channels: &MempoolNodeCommunication,
) -> CommClients {
    let mempool_client: Option<SharedMempoolClient> =
        match config.components.gateway_component.execute {
            true => Some(Arc::new(MempoolClientImpl::new(channels.get_mempool_tx()))),
            false => None,
        };
    CommClients { mempool_client }
}
