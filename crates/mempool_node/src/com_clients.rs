use std::sync::Arc;

use starknet_mempool_types::communication::MempoolClientImpl;

use crate::com_channels::CommChannels;
use crate::config::MempoolNodeConfig;

pub struct CommClients {
    pub mempool_client: Option<Arc<MempoolClientImpl>>,
}

pub fn create_clients(config: &MempoolNodeConfig, channels: &CommChannels) -> CommClients {
    let mempool_client = match config.components.gateway_component.execute {
        true => Some(Arc::new(MempoolClientImpl::new(channels.mempool_channel.tx.clone()))),
        false => None,
    };
    CommClients { mempool_client }
}
