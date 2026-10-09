mod models;
mod sim_status;
mod state;

use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use models::{
    CompletePaymentRequest, CreateDepositRequest, CreateDepositResponse, CreateShopPaymentRequest,
    CreateShopPaymentResponse, ErrorBody, HealthResponse,
};
use sim_status::{SimDirective, SimStatus, HEADER_NAME};
use state::{AppState, PaymentKind};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let host = std::env::var("HTTP_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "18100".into())
        .parse()?;
    let public_base_url =
        std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| format!("http://127.0.0.1:{port}"));
    let callback_signature =
        std::env::var("CALLBACK_SIGNATURE").unwrap_or_else(|_| "test-signature".into());
    let payment_ttl_secs: i64 = std::env::var("PAYMENT_TTL_SECS")
        .unwrap_or_else(|_| "1800".into())
        .parse()?;

    let state = AppState::new(public_base_url, callback_signature, payment_ttl_secs);
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/shop/payment", post(create_shop_payment))
        .route("/v1/shop/payment/{partner_order_no}", get(get_payment))
        .route(
            "/v1/shop/payment/{partner_order_no}/complete",
            post(complete_payment),
        )
        .route("/v1/deposit/payment", post(create_deposit_payment))
        .route("/v1/deposit/payment/{partner_order_no}", get(get_payment))
        .route(
            "/v1/deposit/payment/{partner_order_no}/complete",
            post(complete_payment),
        )
        .route("/v1/payments", get(list_payments))
        .route("/pay/{partner_order_no}", get(checkout_page))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!(
        "payment-gate listening on http://{addr} (header {HEADER_NAME}='STATUS' or 'STATUS ms')"
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        service: "payment-gate".into(),
        timestamp_ms: Utc::now().timestamp_millis(),
    })
}

async fn create_shop_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateShopPaymentRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let directive = SimDirective::from_headers(&headers)?;
    if req.fiat_currency != "HKD" && req.fiat_currency != "USD" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "fiat_currency must be HKD or USD",
        ));
    }
    if req.callback_url.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "callback_url required"));
    }
    let (record, checkout_url) = state
        .create_shop(req)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (sim_status, sim_delay_ms, header_echo) = schedule_payment(&state, &record.partner_order_no, directive);
    Ok((
        StatusCode::CREATED,
        [(HEADER_NAME, header_echo)],
        Json(CreateShopPaymentResponse {
            partner_order_no: record.partner_order_no,
            checkout_url,
            status: "pending".into(),
            sim_status,
            sim_delay_ms,
            expires_at: record.expires_at.unwrap_or_else(Utc::now),
            source: "test".into(),
        }),
    ))
}

async fn create_deposit_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateDepositRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let directive = SimDirective::from_headers(&headers)?;
    if req.amount <= 0.0 {
        return Err(err(StatusCode::BAD_REQUEST, "amount must be > 0"));
    }
    let record = state
        .create_deposit(req)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let (sim_status, sim_delay_ms, header_echo) =
        schedule_payment(&state, &record.partner_order_no, directive);
    Ok((
        StatusCode::CREATED,
        [(HEADER_NAME, header_echo)],
        Json(CreateDepositResponse {
            partner_order_no: record.partner_order_no,
            status: "pending".into(),
            sim_status,
            sim_delay_ms,
            source: record.source,
        }),
    ))
}

async fn get_payment(
    State(state): State<AppState>,
    Path(partner_order_no): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let record = state
        .get(&partner_order_no)
        .await
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "payment not found"))?;
    Ok(Json(record.to_view()))
}

async fn list_payments(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.list().await)
}

async fn complete_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(partner_order_no): Path<String>,
    body: Option<Json<CompletePaymentRequest>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorBody>)> {
    let existing = state
        .get(&partner_order_no)
        .await
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "payment not found"))?;

    if let Some(directive) = SimDirective::from_headers(&headers)? {
        if !directive.schedules_change() {
            return Ok((
                StatusCode::OK,
                [(HEADER_NAME, directive.header_value())],
                Json(existing.to_view_with_sim(directive.status.as_str(), directive.delay_ms)),
            ));
        }
        let (sim_status, sim_delay_ms, header_echo) =
            schedule_payment(&state, &partner_order_no, Some(directive));
        let pending = state
            .get(&partner_order_no)
            .await
            .ok_or_else(|| err(StatusCode::NOT_FOUND, "payment not found"))?;
        return Ok((
            StatusCode::ACCEPTED,
            [(HEADER_NAME, header_echo)],
            Json(pending.to_view_with_sim(&sim_status, sim_delay_ms)),
        ));
    }

    // No header: apply body status immediately (manual complete helper).
    let req = body.map(|b| b.0).unwrap_or(CompletePaymentRequest {
        status: None,
        fire_callback: true,
    });
    let status = req.status.unwrap_or_else(|| match existing.kind {
        PaymentKind::Shop => "paid".into(),
        PaymentKind::Deposit => "success".into(),
    });
    let label = match status.as_str() {
        "paid" | "success" => "SUCCESS",
        "pending" => "PENDING",
        "failed" | "rejected" => "REJECTED",
        "cancelled" | "canceled" => "CANCEL",
        _ => "SUCCESS",
    };
    if status == "pending" {
        return Ok((
            StatusCode::OK,
            [(HEADER_NAME, format!("{label} 0"))],
            Json(existing.to_view_with_sim(label, 0)),
        ));
    }
    let record = state
        .complete(&partner_order_no, &status, req.fire_callback)
        .await
        .map_err(|e| {
            let code = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            err(code, e.to_string())
        })?;
    Ok((
        StatusCode::OK,
        [(HEADER_NAME, format!("{label} 0"))],
        Json(record.to_view_with_sim(label, 0)),
    ))
}

/// Always leave the record PENDING in the HTTP response; schedule final status + webhook.
fn schedule_payment(
    state: &AppState,
    partner_order_no: &str,
    directive: Option<SimDirective>,
) -> (String, u64, String) {
    let Some(directive) = directive.filter(|d| d.schedules_change()) else {
        return (
            SimStatus::Pending.as_str().into(),
            0,
            format!("{} 0", SimStatus::Pending.as_str()),
        );
    };

    let state = state.clone();
    let order = partner_order_no.to_string();
    let delay_ms = directive.delay_ms;
    let target = directive.status;
    tokio::spawn(async move {
        if delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
        let Some(record) = state.get(&order).await else {
            tracing::warn!(%order, "scheduled status skipped; payment gone");
            return;
        };
        if record.status != "pending" {
            tracing::info!(%order, status = %record.status, "scheduled status skipped; already finalized");
            return;
        }
        let domain_status = match record.kind {
            PaymentKind::Shop => target.shop_status(),
            PaymentKind::Deposit => target.deposit_status(),
        };
        match state
            .complete(&order, domain_status, target.should_fire_callback())
            .await
        {
            Ok(_) => tracing::info!(
                %order,
                sim_status = target.as_str(),
                delay_ms,
                "applied scheduled payment status + webhook"
            ),
            Err(e) => tracing::error!(%order, error = %e, "scheduled payment status failed"),
        }
    });

    (
        target.as_str().into(),
        delay_ms,
        directive.header_value(),
    )
}

async fn checkout_page(
    State(state): State<AppState>,
    Path(partner_order_no): Path<String>,
) -> Result<Html<String>, (StatusCode, Json<ErrorBody>)> {
    let record = state
        .get(&partner_order_no)
        .await
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "payment not found"))?;
    let html = format!(
        r#"<!doctype html>
<html><head><meta charset="utf-8"><title>FunnyX payment-gate</title>
<style>
body{{font-family:system-ui,sans-serif;max-width:40rem;margin:2rem auto;padding:0 1rem;background:#0f1419;color:#e7ecf1}}
code,button{{font:inherit}} button{{margin:.25rem .5rem .25rem 0;padding:.5rem .9rem;cursor:pointer}}
.meta{{opacity:.8;font-size:.9rem}} .ok{{color:#7dcea0}}
</style></head><body>
<h1>payment-gate (test)</h1>
<p class="meta">partner_order_no: <code>{order}</code></p>
<p>status: <strong class="ok">{status}</strong> · source: <code>{source}</code></p>
<p>Submit stays PENDING; header <code>X-Sim-Status: STATUS ms</code> schedules change + webhook.</p>
<button onclick="done('SUCCESS 0')">SUCCESS 0</button>
<button onclick="done('SUCCESS 3000')">SUCCESS 3000</button>
<button onclick="done('REJECTED 5000')">REJECTED 5000</button>
<button onclick="done('CANCEL 2000')">CANCEL 2000</button>
<pre id="out"></pre>
<script>
async function done(sim){{
  const r = await fetch('/v1/shop/payment/{order}/complete', {{
    method:'POST',
    headers:{{'content-type':'application/json','X-Sim-Status':sim}},
    body: JSON.stringify({{fire_callback:true}})
  }});
  document.getElementById('out').textContent = await r.text();
}}
</script>
</body></html>"#,
        order = record.partner_order_no,
        status = record.status,
        source = record.source,
    );
    Ok(Html(html))
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.into() }))
}
