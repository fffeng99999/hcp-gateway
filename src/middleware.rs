use axum::{body::Body, extract::Request, middleware::Next, response::Response};

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    tracing::debug!("Request: {} {}", method, uri);

    let response = next.run(request).await;

    let status = response.status();
    tracing::debug!("Response: {} {}", status, uri);

    response
}
