//! HTTP/JSON-транспорт на axum.
//!
//! | Метод | Путь                     | Эндпоинт    | Успех                |
//! |-------|--------------------------|-------------|----------------------|
//! | POST  | `/medicines`             | `create`    | 201 `{"id": "..."}`  |
//! | GET   | `/medicines/{id}`        | `get_by_id` | 200 `Medicine`       |
//! | POST  | `/medicines/{id}/sell`   | `sell`      | 204 без тела         |
//! | GET   | `/health`                | —           | 200 `{"status":"ok"}`|
//!
//! Роль `decodeXRequest` из Go kit выполняют экстракторы axum (`Path`,
//! `Json`), роль `encodeResponse` — реализация `IntoResponse`.

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::pharmacy::endpoint::{
    CreateRequest, CreateResponse, Endpoints, GetByIdRequest, SellRequest,
};
use crate::pharmacy::{Error, Medicine};

/// Собирает HTTP-маршрутизатор поверх набора эндпоинтов.
pub fn router(endpoints: Endpoints) -> Router {
    Router::new()
        .route("/medicines", post(create))
        .route("/medicines/{id}", get(get_by_id))
        .route("/medicines/{id}/sell", post(sell))
        .route("/health", get(|| async { Json(json!({"status": "ok"})) }))
        .with_state(endpoints)
}

#[derive(Debug, Deserialize)]
struct SellBody {
    count: u32,
}

async fn create(
    State(e): State<Endpoints>,
    body: Result<Json<Medicine>, JsonRejection>,
) -> Result<(StatusCode, Json<CreateResponse>), HttpError> {
    let Json(medicine) = body?;
    let resp = (e.create)(CreateRequest { medicine }).await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn get_by_id(
    State(e): State<Endpoints>,
    Path(id): Path<String>,
) -> Result<Json<Medicine>, HttpError> {
    let resp = (e.get_by_id)(GetByIdRequest { id }).await?;
    Ok(Json(resp.medicine))
}

async fn sell(
    State(e): State<Endpoints>,
    Path(id): Path<String>,
    body: Result<Json<SellBody>, JsonRejection>,
) -> Result<StatusCode, HttpError> {
    let Json(SellBody { count }) = body?;
    (e.sell)(SellRequest { id, count }).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Ошибка, отдаваемая клиенту как `{"error": "..."}` с подходящим статусом.
#[derive(Debug)]
pub struct HttpError {
    status: StatusCode,
    message: String,
}

impl From<Error> for HttpError {
    fn from(err: Error) -> Self {
        let status = match err {
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::AlreadyExists(_) | Error::InsufficientStock { .. } => StatusCode::CONFLICT,
            Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
        };
        Self {
            status,
            message: err.to_string(),
        }
    }
}

impl From<JsonRejection> for HttpError {
    fn from(rejection: JsonRejection) -> Self {
        Self {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}
