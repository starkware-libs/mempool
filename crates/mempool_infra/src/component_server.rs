use std::net::{IpAddr, SocketAddr};

use bincode::{deserialize, serialize};
use hyper::body::to_bytes;
use hyper::header::CONTENT_TYPE;
use hyper::service::{make_service_fn, service_fn};
use hyper::{
    Body, Error as HyperError, Request as HyperRequest, Response as HyperResponse, Server,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

use crate::component_definitions::{ComponentRequestAndResponseSender, ComponentRequestHandler};

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

pub struct ComponentServerHttp<Component, Request, Response> {
    socket: SocketAddr,
    component: Component,
    _req: std::marker::PhantomData<Request>,
    _res: std::marker::PhantomData<Response>,
}

impl<Component, Request, Response> ComponentServerHttp<Component, Request, Response>
where
    Request: for<'a> Deserialize<'a> + 'static,
    Response: Serialize + 'static,
    Component: ComponentRequestHandler<Request, Response> + 'static + Clone,
{
    pub fn new(component: Component, ip_address: IpAddr, port: u16) -> Self {
        Self {
            component,
            socket: SocketAddr::new(ip_address, port),
            _req: Default::default(),
            _res: Default::default(),
        }
    }

    pub async fn start(self) {
        let make_svc = make_service_fn(|_conn| {
            let component = self.component.clone();
            async {
                Ok::<_, HyperError>(service_fn(move |req| Self::handler(req, component.clone())))
            }
        });

        Server::bind(&self.socket).serve(make_svc).await.unwrap();
    }

    async fn handler(
        http_request: HyperRequest<Body>,
        mut component: Component,
    ) -> Result<HyperResponse<Body>, HyperError> {
        let body_bytes = to_bytes(http_request.into_body()).await?;
        let component_request: Request =
            deserialize(&body_bytes).expect("Request deserialization should succeed");

        // Scoping is for releasing lock early (otherwise, component is locked until end of
        // function)
        let component_response = component.handle_request(component_request).await;
        let http_response = HyperResponse::builder()
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(
                serialize(&component_response).expect("Response serialization should succeed"),
            ))
            .expect("Response builidng should succeed");

        Ok(http_response)
    }
}
