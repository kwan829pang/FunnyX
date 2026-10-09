//! S2S ingest on SOCKET_PORT (not Client Web).

use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::config::Config;
use crate::ingest::ShopPaymentCallback;
use crate::settle;
use crate::store::EventStore;

pub fn spawn_socket(
    addr: SocketAddr,
    store: EventStore,
    config: Config,
    http: reqwest::Client,
) {
    tokio::spawn(async move {
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                tracing::info!("funnyx-webhook-server socket ingest on {addr}");
                loop {
                    match listener.accept().await {
                        Ok((stream, peer)) => {
                            let store = store.clone();
                            let config = config.clone();
                            let http = http.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_conn(stream, store, config, http).await {
                                    tracing::warn!(%peer, "socket ingest closed: {e}");
                                }
                            });
                        }
                        Err(e) => tracing::warn!("socket accept: {e}"),
                    }
                }
            }
            Err(e) => tracing::error!("socket bind {addr}: {e}"),
        }
    });
}

async fn handle_conn(
    mut stream: tokio::net::TcpStream,
    store: EventStore,
    config: Config,
    http: reqwest::Client,
) -> anyhow::Result<()> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        loop {
            match funnyx_socket::decode_frame(&buf).map_err(|e| anyhow::anyhow!("{e}"))? {
                None => break,
                Some((body, consumed)) => {
                    buf.drain(..consumed);
                    if let Some(pong) = funnyx_heartbeat::pong_frame_if_ping(&body) {
                        stream.write_all(&pong).await?;
                        continue;
                    }
                    match parse_ingest(&body) {
                        Ok(cb) => {
                            let out = settle::accept_shop_payment(
                                &store, &config, &http, cb, "socket",
                            )
                            .await;
                            let reply = match out {
                                Ok(v) => serde_json::to_vec(&v).unwrap_or_else(|_| b"{}".to_vec()),
                                Err(e) => serde_json::to_vec(&serde_json::json!({"error": e}))
                                    .unwrap_or_else(|_| b"{}".to_vec()),
                            };
                            stream
                                .write_all(&funnyx_socket::encode_frame(&reply))
                                .await?;
                        }
                        Err(e) => {
                            let reply = serde_json::json!({"error": e});
                            let bytes = serde_json::to_vec(&reply).unwrap_or_default();
                            stream
                                .write_all(&funnyx_socket::encode_frame(&bytes))
                                .await?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn parse_ingest(raw: &[u8]) -> Result<ShopPaymentCallback, String> {
    let json = match funnyx_socket::decompress_client_json(raw) {
        Ok(s) => s,
        Err(_) => String::from_utf8(raw.to_vec()).map_err(|e| e.to_string())?,
    };
    let v: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let event_type = v
        .get("event_type")
        .and_then(|x| x.as_str())
        .unwrap_or("shop_payment");
    if event_type != "shop_payment" {
        return Err(format!("unsupported event_type: {event_type}"));
    }
    if let Some(body) = v.get("body") {
        return ShopPaymentCallback::from_value(body);
    }
    ShopPaymentCallback::from_value(&v)
}
