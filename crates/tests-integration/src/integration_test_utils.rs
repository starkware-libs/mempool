use std::future::Future;

use hyper::StatusCode;
use reqwest::Response;

// TODO: change expected type to something more concrete once add_tx is fully implemented.
pub async fn check_success(request: impl Future<Output = Response>, expected: impl AsRef<[u8]>) {
    let response = check_request(request, StatusCode::OK).await.bytes().await.unwrap();

    assert_eq!(response, expected.as_ref())
}

pub async fn check_request(
    request: impl Future<Output = Response>,
    status_code: StatusCode,
) -> Response {
    let response = request.await;

    assert_eq!(response.status(), status_code);
    response
}
