//! Reverse-proxy forwarding to upstream services.

use std::sync::atomic::AtomicUsize;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::IntoResponse;
use axum::Json;
use bytes::Bytes;
use reqwest::Client;

use crate::auth::UpstreamTarget;
use crate::config::Config;
use crate::memory::{ServiceRole, SharedGatewayMemory};
use crate::sts::ErrorBody;

pub struct RrCounters {
    pub client_center: AtomicUsize,
    pub message_center: AtomicUsize,
    pub admin_api: AtomicUsize,
}

impl Default for RrCounters {
    fn default() -> Self {
        Self {
            client_center: AtomicUsize::new(0),
            message_center: AtomicUsize::new(0),
            admin_api: AtomicUsize::new(0),
        }
    }
}

pub async fn resolve_upstream(
    memory: &SharedGatewayMemory,
    config: &Config,
    target: UpstreamTarget,
    rr: &RrCounters,
) -> String {
    let (role, fallback, counter) = match target {
        UpstreamTarget::ClientCenter => (
            ServiceRole::ClientCenter,
            config.client_center_url.as_str(),
            &rr.client_center,
        ),
        UpstreamTarget::MessageCenter => (
            ServiceRole::MessageCenter,
            config.message_center_url.as_str(),
            &rr.message_center,
        ),
        UpstreamTarget::AdminApi => (
            ServiceRole::AdminApi,
            config.admin_api_url.as_str(),
            &rr.admin_api,
        ),
    };
    let map = memory.read().await;
    if let Some(node) = map.pick_healthy(role, counter) {
        return node.http_url.clone();
    }
    fallback.trim_end_matches('/').to_string()
}

pub async fn forward(
    http: &Client,
    upstream_base: &str,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
    extra_headers: Vec<(HeaderName, HeaderValue)>,
) -> axum::response::Response {
    let path_and_query = uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or(uri.path());
    let url = format!(
        "{}{}",
        upstream_base.trim_end_matches('/'),
        path_and_query
    );

    let mut builder = http.request(
        reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap_or(reqwest::Method::GET),
        &url,
    );

    for (name, value) in headers.iter() {
        if is_hop_by_hop(name.as_str()) {
            continue;
        }
        if let Ok(v) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
            builder = builder.header(name.as_str(), v);
        }
    }
    for (name, value) in extra_headers {
        if let Ok(v) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
            builder = builder.header(name.as_str(), v);
        }
    }

    if !body.is_empty() {
        builder = builder.body(body.to_vec());
    }

    match builder.send().await {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let mut out_headers = HeaderMap::new();
            for (name, value) in resp.headers().iter() {
                if is_hop_by_hop(name.as_str()) {
                    continue;
                }
                if let (Ok(n), Ok(v)) = (
                    HeaderName::from_bytes(name.as_str().as_bytes()),
                    HeaderValue::from_bytes(value.as_bytes()),
                ) {
                    out_headers.insert(n, v);
                }
            }
            match resp.bytes().await {
                Ok(bytes) => {
                    let mut response = Body::from(bytes).into_response();
                    *response.status_mut() = status;
                    *response.headers_mut() = out_headers;
                    response
                }
                Err(e) => err_response(
                    StatusCode::BAD_GATEWAY,
                    format!("upstream body: {e}"),
                ),
            }
        }
        Err(e) => err_response(StatusCode::BAD_GATEWAY, format!("upstream: {e}")),
    }
}

fn is_hop_by_hop(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
            | "host"
            | "content-length"
    )
}

pub fn err_response(status: StatusCode, message: impl Into<String>) -> axum::response::Response {
    let body = Json(ErrorBody {
        error: message.into(),
    });
    (status, body).into_response()
}
