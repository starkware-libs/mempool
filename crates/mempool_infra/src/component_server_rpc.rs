use std::net::{IpAddr, SocketAddr};

use async_trait::async_trait;

#[async_trait]
pub trait ServerStart {
    async fn start_server(self, address: SocketAddr);
}

pub struct ComponentServerRpc<Component> {
    component: Option<Component>,
    address: SocketAddr,
}

impl<Component: ServerStart> ComponentServerRpc<Component> {
    pub fn new(component: Component, ip_address: IpAddr, port: u16) -> Self {
        Self { component: Some(component), address: SocketAddr::new(ip_address, port) }
    }

    pub async fn start(&mut self) {
        self.component.take().unwrap().start_server(self.address).await;
    }
}
