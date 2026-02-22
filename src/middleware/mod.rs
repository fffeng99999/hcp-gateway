use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{header, StatusCode},
};
use tower_http::request_id::RequestId;
use crate::utils::auth::verify_token;

pub async fn propagate_request_id(request: Request, next: Next) -> Response {
    let request_id = request.extensions().get::<RequestId>().map(|id| id.header_value().clone());
    let mut response = next.run(request).await;
    if let Some(request_id) = request_id {
        response.headers_mut().insert("x-request-id", request_id);
    }
    response
}

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    tracing::debug!("Request: {} {}", method, uri);

    let response = next.run(request).await;

    let status = response.status();
    tracing::debug!("Response: {} {}", status, uri);

    response
}

pub async fn auth_middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request.headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];

    match verify_token(token) {
        Ok(claims) => {
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
