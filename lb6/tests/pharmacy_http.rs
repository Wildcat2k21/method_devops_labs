//! Задание 2: HTTP-транспорт «Аптеки» без сети — запросы подаются прямо
//! в маршрутизатор через `tower::ServiceExt::oneshot`.

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use lb6::pharmacy::transport::http::router;
use lb6::pharmacy::{Endpoints, InMemoryRepository, Pharmacy};
use serde_json::{Value, json};
use tower::ServiceExt;

fn app() -> Router {
    router(Endpoints::new(Pharmacy::new(InMemoryRepository::default())))
}

async fn call(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(v) => {
            req = req.header(header::CONTENT_TYPE, "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let resp = app.clone().oneshot(req.body(body).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

fn paracetamol() -> Value {
    json!({
        "name": "Парацетамол",
        "manufacturer": "Фармстандарт",
        "price": 45.9,
        "prescription": false,
        "quantity": 100
    })
}

#[tokio::test]
async fn full_scenario() {
    let app = app();

    let (status, created) = call(&app, "POST", "/medicines", Some(paracetamol())).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().unwrap().to_owned();

    let (status, got) = call(&app, "GET", &format!("/medicines/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(got["name"], "Парацетамол");
    assert_eq!(got["quantity"], 100);

    let (status, _) = call(
        &app,
        "POST",
        &format!("/medicines/{id}/sell"),
        Some(json!({"count": 30})),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, got) = call(&app, "GET", &format!("/medicines/{id}"), None).await;
    assert_eq!(got["quantity"], 70);

    let (status, err) = call(
        &app,
        "POST",
        &format!("/medicines/{id}/sell"),
        Some(json!({"count": 71})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(err["error"].as_str().unwrap().contains("в наличии 70"));
}

#[tokio::test]
async fn error_mapping() {
    let app = app();

    let (status, _) = call(&app, "GET", "/medicines/unknown", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let mut bad = paracetamol();
    bad["price"] = json!(-1);
    let (status, err) = call(&app, "POST", "/medicines", Some(bad)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(err["error"].as_str().unwrap().contains("price"));

    // Отрицательное количество отсекается при десериализации в u32.
    let (status, err) = call(
        &app,
        "POST",
        "/medicines/x/sell",
        Some(json!({"count": -5})),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(err["error"].is_string());

    let mut with_id = paracetamol();
    with_id["id"] = json!("pcm-500");
    let (status, _) = call(&app, "POST", "/medicines", Some(with_id.clone())).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = call(&app, "POST", "/medicines", Some(with_id)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn health() {
    let (status, body) = call(&app(), "GET", "/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"status": "ok"}));
}
