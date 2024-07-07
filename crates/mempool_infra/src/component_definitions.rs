use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait ComponentRequestHandler<Request, Response> {
    async fn handle_request(&mut self, request: Request) -> Response;
}

pub struct ComponentRequestAndResponseSender<Request, Response>
where
    Request: Send + Sync,
    Response: Send + Sync,
{
    pub request: Request,
    pub tx: Sender<Response>,
}

pub const APPLICATION_OCTET_STREAM: &str = "application/octet-stream";
pub const X_INTERNAL_ERROR_REASON_HEADER: &str = "x-internal-error-reason";
pub const DESERIALIZAION_FAILED_REASON: &str = "Deserialization failed";
