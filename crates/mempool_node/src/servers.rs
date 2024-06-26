use starknet_gateway::communication::create_gateway_server;
use starknet_mempool::communication::create_mempool_server;
use starknet_mempool_infra::component_server::ComponentServerStarter;

use crate::communication::MempoolNodeCommunication;
use crate::components::Components;
use crate::config::MempoolNodeConfig;

pub struct Servers {
    pub gateway: Option<Box<dyn ComponentServerStarter>>,
    pub mempool: Option<Box<dyn ComponentServerStarter>>,
}

pub fn create_servers(
    config: &MempoolNodeConfig,
    mut communication: MempoolNodeCommunication,
    components: Components,
) -> Servers {
    let gateway_server: Option<Box<dyn ComponentServerStarter>> =
        if config.components.gateway_component.execute {
            Some(Box::new(create_gateway_server(
                components.gateway.expect("Gateway is not initialized."),
            )))
        } else {
            None
        };

    let mempool_server: Option<Box<dyn ComponentServerStarter>> =
        if config.components.mempool_component.execute {
            Some(Box::new(create_mempool_server(
                components.mempool.expect("Mempool is not initialized."),
                communication.take_mempool_rx(),
            )))
        } else {
            None
        };

    Servers { gateway: gateway_server, mempool: mempool_server }
}

// TODO (Lev): Implement the run server components function.
