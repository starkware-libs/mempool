mod common;

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use async_trait::async_trait;
use common::{ComponentATrait, ComponentBTrait};
use hyper::header::CONTENT_TYPE;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
use serde::{Deserialize, Serialize};
use starknet_mempool_infra::component_definitions::ComponentRequestHandler;
use tokio::sync::Mutex;
use tokio::task;

use crate::common::{ComponentA, ComponentB, ValueA, ValueB};

// todo(uriel): move to common
#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentARequest {
    AGetValue,
}

// todo(uriel): move to common
#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentAResponse {
    Value(ValueA),
}

// todo(uriel): make generic - ComponentClientHttp<Component>
struct ComponentAClientHttp {
    uri: Uri,
}

impl ComponentAClientHttp {
    pub fn new(ip_address: IpAddr, port: u16) -> Self {
        let uri = match ip_address {
            IpAddr::V4(ip_address) => format!("http://{}:{}/", ip_address, port).parse().unwrap(),
            IpAddr::V6(ip_address) => format!("http://[{}]:{}/", ip_address, port).parse().unwrap(),
        };
        Self { uri }
    }
}

// todo(uriel): change the component trait to client specific and make it return result
#[async_trait]
impl ComponentATrait for ComponentAClientHttp {
    async fn a_get_value(&self) -> ValueA {
        let component_request = ComponentARequest::AGetValue;
        let http_request = Request::post(self.uri.clone())
            .header("Content-Type", "application/octet-stream")
            .body(Body::from(
                bincode::serialize(&component_request)
                    .expect("Request serialization should succeed"),
            ))
            .expect("Request builidng should succeed");

        // todo(uriel): add configuration to control number of retries
        let http_response =
            Client::new().request(http_request).await.expect("Could not connect to server");
        let body_bytes = hyper::body::to_bytes(http_response.into_body())
            .await
            .expect("Could not get response from server");
        match bincode::deserialize(&body_bytes).expect("Response Deserialization should succeed") {
            ComponentAResponse::Value(value) => value,
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<ComponentARequest, ComponentAResponse> for ComponentA {
    async fn handle_request(&mut self, request: ComponentARequest) -> ComponentAResponse {
        match request {
            ComponentARequest::AGetValue => ComponentAResponse::Value(self.a_get_value().await),
        }
    }
}

struct ComponentAServerHttp {
    socket: SocketAddr,
    component: Arc<Mutex<ComponentA>>,
}

impl ComponentAServerHttp {
    pub fn new(component: ComponentA, ip_address: IpAddr, port: u16) -> Self {
        Self {
            component: Arc::new(Mutex::new(component)),
            socket: SocketAddr::new(ip_address, port),
        }
    }

    pub async fn start(&mut self) {
        let make_svc = make_service_fn(|_conn| {
            let component = Arc::clone(&self.component);
            async {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    Self::handler(req, Arc::clone(&component))
                }))
            }
        });

        Server::bind(&self.socket.clone()).serve(make_svc).await.unwrap();
    }

    async fn handler(
        http_request: Request<Body>,
        component: Arc<Mutex<ComponentA>>,
    ) -> Result<Response<Body>, hyper::Error> {
        let body_bytes = hyper::body::to_bytes(http_request.into_body()).await?;
        let component_request: ComponentARequest =
            bincode::deserialize(&body_bytes).expect("Request Deserialization should succeed");

        // scoping is for releasing lock early (otherwise, component is locked until end of
        // function)
        let component_response;
        {
            let mut component_guard = component.lock().await;
            component_response = component_guard.handle_request(component_request).await;
        }
        let http_response = Response::builder()
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(
                bincode::serialize(&component_response)
                    .expect("Response Serialization should succeed"),
            ))
            .expect("Response builidng should succeed");

        Ok(http_response)
    }
}

// todo(uriel): move to common
#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentBRequest {
    BGetValue,
}

// todo(uriel): move to common
#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentBResponse {
    Value(ValueB),
}

// todo(uriel): make generic - ComponentClientHttp<Component>
struct ComponentBClientHttp {
    uri: Uri,
}

impl ComponentBClientHttp {
    pub fn new(ip_address: IpAddr, port: u16) -> Self {
        let uri = match ip_address {
            IpAddr::V4(ip_address) => format!("http://{}:{}/", ip_address, port).parse().unwrap(),
            IpAddr::V6(ip_address) => format!("http://[{}]:{}/", ip_address, port).parse().unwrap(),
        };
        Self { uri }
    }
}

// todo(uriel): change the component trait to client specific and make it return result
#[async_trait]
impl ComponentBTrait for ComponentBClientHttp {
    async fn b_get_value(&self) -> ValueB {
        let component_request = ComponentBRequest::BGetValue;
        let http_request = Request::post(self.uri.clone())
            .header("Content-Type", "application/octet-stream")
            .body(Body::from(
                bincode::serialize(&component_request)
                    .expect("Request serialization should succeed"),
            ))
            .expect("Request builidng should succeed");

        // todo(uriel): add configuration to  control number of retries
        let http_response =
            Client::new().request(http_request).await.expect("Could not connect to server");
        let body_bytes = hyper::body::to_bytes(http_response.into_body())
            .await
            .expect("Could not get response from server");
        match bincode::deserialize(&body_bytes).expect("Response Deserialization should succeed") {
            ComponentBResponse::Value(value) => value,
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<ComponentBRequest, ComponentBResponse> for ComponentB {
    async fn handle_request(&mut self, request: ComponentBRequest) -> ComponentBResponse {
        match request {
            ComponentBRequest::BGetValue => ComponentBResponse::Value(self.b_get_value().await),
        }
    }
}

struct ComponentBServerHttp {
    socket: SocketAddr,
    component: Arc<Mutex<ComponentB>>,
}

impl ComponentBServerHttp {
    pub fn new(component: ComponentB, ip_address: IpAddr, port: u16) -> Self {
        Self {
            component: Arc::new(Mutex::new(component)),
            socket: SocketAddr::new(ip_address, port),
        }
    }

    pub async fn start(&mut self) {
        let make_svc = make_service_fn(|_conn| {
            let component = Arc::clone(&self.component);
            async {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    Self::handler(req, Arc::clone(&component))
                }))
            }
        });

        Server::bind(&self.socket.clone()).serve(make_svc).await.unwrap();
    }

    async fn handler(
        http_request: Request<Body>,
        component: Arc<Mutex<ComponentB>>,
    ) -> Result<Response<Body>, hyper::Error> {
        let body_bytes = hyper::body::to_bytes(http_request.into_body()).await?;
        let component_request: ComponentBRequest =
            bincode::deserialize(&body_bytes).expect("Request Deserialization should succeed");

        // scoping is for releasing lock early (otherwise, component is locked until end of
        // function)
        let component_response;
        {
            let mut component_guard = component.lock().await;
            component_response = component_guard.handle_request(component_request).await;
        }
        let http_response = Response::builder()
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(bincode::serialize(&component_response).unwrap()))
            .expect("Response builidng should succeed");

        Ok(http_response)
    }
}

async fn verify_response(ip_address: IpAddr, port: u16, expected_value: ValueA) {
    let a_client = ComponentAClientHttp::new(ip_address, port);
    assert_eq!(a_client.a_get_value().await, expected_value);
}

#[tokio::test]
async fn test_setup() {
    let setup_value: ValueB = 90;
    let expected_value: ValueA = setup_value.into();

    let local_ip = "::1".parse().unwrap();
    let a_port = 10000;
    let b_port = 10001;

    let a_client = ComponentAClientHttp::new(local_ip, a_port);
    let b_client = ComponentBClientHttp::new(local_ip, b_port);

    let component_a = ComponentA::new(Box::new(b_client));
    let component_b = ComponentB::new(setup_value, Box::new(a_client));

    let mut component_a_server = ComponentAServerHttp::new(component_a, local_ip, a_port);
    let mut component_b_server = ComponentBServerHttp::new(component_b, local_ip, b_port);

    task::spawn(async move {
        component_a_server.start().await;
    });

    task::spawn(async move {
        component_b_server.start().await;
    });

    task::yield_now().await;

    verify_response(local_ip, a_port, expected_value).await;
}
