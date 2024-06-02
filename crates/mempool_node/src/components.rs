use starknet_gateway::gateway::Gateway;
use starknet_mempool::mempool::Mempool;
use starknet_mempool_infra::component_runner::ComponentCreator;
use starknet_mempool_infra::empty_config::EmptyConfig;

use crate::com_clients::CommClients;
use crate::config::MempoolNodeConfig;

pub struct Components {
    pub gateway: Option<Gateway>,
    pub mempool: Option<Mempool>,
}

pub fn create_components(config: &MempoolNodeConfig, clients: &CommClients) -> Components {
    let mut components = Components { gateway: None, mempool: None };
    if config.components.gateway_component.execute {
        components.gateway =
            Some(Gateway::create(config.gateway_config.clone(), clients.mempool_client.clone()));
    }
    if config.components.mempool_component.execute {
        components.mempool = Some(Mempool::create(EmptyConfig, None));
    }
    components
}
