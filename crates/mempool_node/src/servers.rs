// use std::future::pending;
// use std::pin::Pin;

// use futures::{Future, FutureExt};
use starknet_gateway::gateway::create_gateway_server;
use starknet_mempool::communication::create_mempool_server;
use starknet_mempool_infra::component_server::CommunicationServer;

use crate::communication::MempoolNodeCommunication;
use crate::components::Components;
use crate::config::MempoolNodeConfig;

pub struct Servers {
    pub gateway: Option<Box<dyn CommunicationServer>>,
    pub mempool: Option<Box<dyn CommunicationServer>>,
}

pub fn create_servers(
    config: &MempoolNodeConfig,
    mut channels: MempoolNodeCommunication,
    components: Components,
) -> Servers {
    let mut servers = Servers { gateway: None, mempool: None };

    if config.components.gateway_component.execute {
        servers.gateway = Some(Box::new(create_gateway_server(
            components.gateway.expect("Gateway component is not initialized."),
        )));
    }

    if config.components.mempool_component.execute {
        servers.mempool = Some(Box::new(create_mempool_server(
            components.mempool.expect("Mempool component is not initialized."),
            channels.take_mempool_rx(),
        )));
    }

    servers
}

// TODO (Lev): Implement the run server components function.
