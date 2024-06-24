use tokio::sync::mpsc::Sender;

pub struct ComponentRequestAndResponseSender<Request, Response>
where
    Request: Send + Sync,
    Response: Send + Sync,
{
    pub request: Request,
    pub tx: Sender<Response>,
}
