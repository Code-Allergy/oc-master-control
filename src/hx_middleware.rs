use crate::root;
use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use http::header::CONTENT_LENGTH;
use maud::PreEscaped;

pub async fn hx_response_middleware(request: Request<Body>, next: Next) -> Response {
    let request_uri = request.uri();
    let is_htmx = request
        .headers()
        .get("HX-Request")
        .is_some_and(|h| h.to_str().is_ok_and(|v| v == "true"));

    let is_static = request_uri.to_string().eq("/favicon.ico")
        || request_uri.to_string().starts_with("/static");

    let is_api = request_uri.to_string().starts_with("/api");

    let mut response = next.run(request).await;

    if !is_htmx && !is_static && !is_api {
        let (mut response_parts, response_body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(response_body, usize::MAX)
            .await
            .unwrap_or_default();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
        let new_body = root(PreEscaped(body_str));
        let new_len = new_body.len();

        // todo note that this strips off any existing headers
        let mut response = Response::builder()
            .status(response_parts.status)
            .header(CONTENT_LENGTH, new_len)
            .body(Body::new(new_body))
            .unwrap();

        response
    } else {
        response
    }
}
