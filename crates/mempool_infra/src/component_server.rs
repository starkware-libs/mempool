use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use async_trait::async_trait;
use bincode::{deserialize, serialize};
use hyper::body::to_bytes;
use hyper::header::CONTENT_TYPE;
use hyper::service::{make_service_fn, service_fn};
use hyper::{
    Body, Error as HyperError, Request as HyperRequest, Response as HyperResponse, Server,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;

use crate::component_definitions::ComponentRequestAndResponseSender;

#[async_trait]
pub trait ComponentRequestHandler<Request, Response> {
    async fn handle_request(&mut self, request: Request) -> Response;
}

pub struct ComponentServer<Component, Request, Response>
where
    Component: ComponentRequestHandler<Request, Response>,
    Request: Send + Sync,
    Response: Send + Sync,
{
    component: Component,
    rx: Receiver<ComponentRequestAndResponseSender<Request, Response>>,
}

impl<Component, Request, Response> ComponentServer<Component, Request, Response>
where
    Component: ComponentRequestHandler<Request, Response>,
    Request: Send + Sync,
    Response: Send + Sync,
{
    pub fn new(
        component: Component,
        rx: Receiver<ComponentRequestAndResponseSender<Request, Response>>,
    ) -> Self {
        Self { component, rx }
    }

    pub async fn start(&mut self) {
        while let Some(request_and_res_tx) = self.rx.recv().await {
            let request = request_and_res_tx.request;
            let tx = request_and_res_tx.tx;

            let res = self.component.handle_request(request).await;

            tx.send(res).await.expect("Response connection should be open.");
        }
    }
}

pub struct ComponentServerHttp<Component> {
    socket: SocketAddr,
    component: Arc<Mutex<Component>>,
}

impl<Component> ComponentServerHttp<Component> {
    pub fn new(component: Component, ip_address: IpAddr, port: u16) -> Self {
        Self {
            component: Arc::new(Mutex::new(component)),
            socket: SocketAddr::new(ip_address, port),
        }
    }

    pub async fn start<Request, Response>(&mut self)
    where
        Request: for<'a> Deserialize<'a> + Send + 'static,
        Response: Serialize + 'static,
        Component: ComponentRequestHandler<Request, Response> + Send + 'static,
    {
        let make_svc = make_service_fn(|_conn| {
            let component = Arc::clone(&self.component);
            async {
                Ok::<_, HyperError>(service_fn(move |req| {
                    Self::handler::<Request, Response>(req, Arc::clone(&component))
                }))
            }
        });

        Server::bind(&self.socket.clone()).serve(make_svc).await.unwrap();
    }

    async fn handler<Request, Response>(
        http_request: HyperRequest<Body>,
        component: Arc<Mutex<Component>>,
    ) -> Result<HyperResponse<Body>, HyperError>
    where
        Request: for<'a> Deserialize<'a>,
        Response: Serialize,
        Component: ComponentRequestHandler<Request, Response>,
    {
        let body_bytes = to_bytes(http_request.into_body()).await?;
        let component_request: Request =
            deserialize(&body_bytes).expect("Request deserialization should succeed");

        // Scoping is for releasing lock early (otherwise, component is locked until end of
        // function)
        let component_response;
        {
            let mut component_guard = component.lock().await;
            component_response = component_guard.handle_request(component_request).await;
        }
        let http_response = HyperResponse::builder()
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(
                serialize(&component_response).expect("Response serialization should succeed"),
            ))
            .expect("Response builidng should succeed");

        Ok(http_response)
    }
}
