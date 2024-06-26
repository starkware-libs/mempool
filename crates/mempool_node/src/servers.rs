use std::future::pending;
use std::pin::Pin;

use futures::{Future, FutureExt};
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

pub async fn run_server_components(
    config: &MempoolNodeConfig,
    servers: Servers,
) -> anyhow::Result<()> {
    // Gateway component.
    // let gateway: Gateway;
    let gateway_future =
        get_server_future("Gateway", config.components.gateway_component.execute, servers.gateway);

    // Mempool component.
    let mempool_future =
        get_server_future("Mempool", config.components.mempool_component.execute, servers.mempool);

    let gateway_handle = tokio::spawn(gateway_future);
    let mempool_handle = tokio::spawn(mempool_future);

    tokio::select! {
        res = gateway_handle => {
            println!("Error: Gateway Server stopped.");
            res?
        }
        res = mempool_handle => {
            println!("Error: Mempool Server stopped.");
            res?
        }
    };
    println!("Error: Servers ended with unexpected Ok.");

    Ok(())
}

pub fn get_server_future(
    name: &str,
    execute_flag: bool,
    server: Option<Box<dyn CommunicationServer>>,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    let server_future = match execute_flag {
        true => {
            let mut server = match server {
                Some(server) => server,
                _ => panic!("{} component is not initialized.", name),
            };
            async move { server.start().await }.boxed()
        }
        false => pending().boxed(),
    };
    server_future
}
