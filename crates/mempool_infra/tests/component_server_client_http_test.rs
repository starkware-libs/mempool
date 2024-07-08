mod common;

use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use async_trait::async_trait;
use bincode::serialize;
use common::{ComponentAClientTrait, ComponentBClientTrait, ResultA, ResultB};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};
use starknet_mempool_infra::component_client::ComponentClientHttp;
use starknet_mempool_infra::component_definitions::{ComponentRequestHandler, ServerError};
use starknet_mempool_infra::component_server::ComponentServerHttp;
use tokio::task;

type ComponentAClient = ComponentClientHttp<ComponentARequest, ComponentAResponse>;
type ComponentBClient = ComponentClientHttp<ComponentBRequest, ComponentBResponse>;

use crate::common::{ComponentA, ComponentB, ValueA, ValueB};

const LOCAL_IP: IpAddr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));

// Todo(uriel): Move to common
#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentARequest {
    AGetValue,
}

// Todo(uriel): Move to common
#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentAResponse {
    Value(ValueA),
}

#[async_trait]
impl ComponentAClientTrait for ComponentClientHttp<ComponentARequest, ComponentAResponse> {
    async fn a_get_value(&self) -> ResultA {
        match self.send(ComponentARequest::AGetValue).await? {
            ComponentAResponse::Value(value) => Ok(value),
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

// Todo(uriel): Move to common
#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentBRequest {
    BGetValue,
}

// Todo(uriel): Move to common
#[derive(Serialize, Deserialize, Debug)]
pub enum ComponentBResponse {
    Value(ValueB),
}

#[async_trait]
impl ComponentBClientTrait for ComponentClientHttp<ComponentBRequest, ComponentBResponse> {
    async fn b_get_value(&self) -> ResultB {
        match self.send(ComponentBRequest::BGetValue).await? {
            ComponentBResponse::Value(value) => Ok(value),
        }
    }
}

#[async_trait]
impl ComponentRequestHandler<ComponentBRequest, ComponentBResponse> for ComponentB {
    async fn handle_request(&mut self, request: ComponentBRequest) -> ComponentBResponse {
        match request {
            ComponentBRequest::BGetValue => ComponentBResponse::Value(self.b_get_value()),
        }
    }
}

async fn verify_response(a_client: ComponentAClient, expected_value: ValueA) {
    assert_eq!(a_client.a_get_value().await.unwrap(), expected_value);
}

#[tokio::test]
async fn test_setup() {
    let setup_value: ValueB = 90;
    let expected_value: ValueA = setup_value.into();

    let a_port = 10000;
    let b_port = 10001;

    let a_client = ComponentAClient::new(LOCAL_IP, a_port);
    let b_client = ComponentBClient::new(LOCAL_IP, b_port);

    let component_a = ComponentA::new(Box::new(b_client));
    let component_b = ComponentB::new(setup_value, Box::new(a_client.clone()));

    let mut component_a_server = ComponentServerHttp::<
        ComponentA,
        ComponentARequest,
        ComponentAResponse,
    >::new(component_a, LOCAL_IP, a_port);
    let mut component_b_server = ComponentServerHttp::<
        ComponentB,
        ComponentBRequest,
        ComponentBResponse,
    >::new(component_b, LOCAL_IP, b_port);

    task::spawn(async move {
        component_a_server.start().await;
    });

    task::spawn(async move {
        component_b_server.start().await;
    });

    // Todo(uriel): Get rid of this
    task::yield_now().await;

    verify_response(a_client.clone(), expected_value).await;
}

async fn verify_error(a_client: ComponentAClient, expected_error_contained_keywords: Vec<&str>) {
    let Err(error) = a_client.a_get_value().await else {
        panic!("Expected an error.");
    };

    for expected_keyword in expected_error_contained_keywords {
        if !error.to_string().contains(expected_keyword) {
            panic!("Expected keyword: '{expected_keyword}' is not in error: '{error}'.")
        }
    }
}

#[tokio::test]
async fn test_unconnected_server() {
    let port = 10002;
    let client = ComponentAClient::new(LOCAL_IP, port);

    let expected_error_contained_keywords = vec!["Connection refused"];
    verify_error(client.clone(), expected_error_contained_keywords).await;
}

#[tokio::test]
async fn test_faulty_server_1() {
    let port = 10003;
    const MOCK_SERVER_ERROR: &str = "Mock server error";

    task::spawn(async move {
        async fn handler(_http_request: Request<Body>) -> Result<Response<Body>, hyper::Error> {
            let server_error =
                ServerError::RequestDeserializationFailure(MOCK_SERVER_ERROR.to_string());
            let http_response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(serialize(&server_error).unwrap()))
                .unwrap();
            Ok(http_response)
        }

        let socket = SocketAddr::new(LOCAL_IP, port);
        let make_svc =
            make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(handler)) });
        Server::bind(&socket).serve(make_svc).await.unwrap();
    });
    task::yield_now().await;

    let client = ComponentAClient::new(LOCAL_IP, port);
    let expected_error_contained_keywords =
        vec![StatusCode::BAD_REQUEST.as_str(), MOCK_SERVER_ERROR];
    verify_error(client.clone(), expected_error_contained_keywords).await;
}

#[tokio::test]
async fn test_faulty_server_2() {
    let port = 10004;

    task::spawn(async move {
        async fn handler(_http_request: Request<Body>) -> Result<Response<Body>, hyper::Error> {
            let garbage = "Garbage";
            let http_response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(serialize(&garbage).unwrap()))
                .unwrap();
            Ok(http_response)
        }

        let socket = SocketAddr::new(LOCAL_IP, port);
        let make_svc =
            make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(handler)) });
        Server::bind(&socket).serve(make_svc).await.unwrap();
    });
    task::yield_now().await;

    let client = ComponentAClient::new(LOCAL_IP, port);
    let expected_error_contained_keywords = vec!["Could not deserialize server response"];
    verify_error(client, expected_error_contained_keywords).await;
}
