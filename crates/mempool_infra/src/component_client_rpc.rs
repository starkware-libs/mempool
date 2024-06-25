use std::net::IpAddr;

fn construct_url(ip_address: IpAddr, port: u16) -> String {
    match ip_address {
        IpAddr::V4(ip_address) => format!("http://{}:{}/", ip_address, port),
        IpAddr::V6(ip_address) => format!("http://[{}]:{}/", ip_address, port),
    }
}

pub struct ComponentClientRpc<Component> {
    pub dst: String,
    _component: std::marker::PhantomData<Component>,
}

impl<Component> ComponentClientRpc<Component> {
    pub fn new(ip_address: IpAddr, port: u16) -> Self {
        Self { dst: construct_url(ip_address, port), _component: Default::default() }
    }
}
