use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{header, StatusCode},
};
use crate::utils::auth::verify_token;

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

    // Compatibility for frontend mock token
    if token.starts_with("mock_token_") {
        let claims = crate::models::Claims {
            sub: "admin".to_string(),
            role: "admin".to_string(),
            exp: usize::MAX,
        };
        request.extensions_mut().insert(claims);
        return Ok(next.run(request).await);
    }

    match verify_token(token) {
        Ok(claims) => {
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
