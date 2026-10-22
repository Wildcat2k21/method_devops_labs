//! Слой эндпоинтов.
//!
//! В Go kit эндпоинт — это функция `func(ctx, request) (response, error)`,
//! которая не знает ничего о транспорте. Здесь то же самое выражено типом
//! [`Endpoint`]: асинхронная функция «запрос → ответ», упакованная в `Arc`,
//! чтобы её можно было разделять между HTTP- и gRPC-транспортами.

use std::pin::Pin;
use std::sync::Arc;

use serde::Serialize;

use super::error::Error;
use super::model::Medicine;
use super::service::PharmacyService;

/// Упакованная `Send`-future.
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

/// Эндпоинт — аналог `endpoint.Endpoint` из Go kit.
pub type Endpoint<Req, Resp> = Arc<dyn Fn(Req) -> BoxFuture<Result<Resp, Error>> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct CreateRequest {
    pub medicine: Medicine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateResponse {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct GetByIdRequest {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct GetByIdResponse {
    pub medicine: Medicine,
}

#[derive(Debug, Clone)]
pub struct SellRequest {
    pub id: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SellResponse;

/// Создаёт эндпоинт из асинхронной функции над сервисом.
fn endpoint<S, Req, Resp, F, Fut>(svc: Arc<S>, f: F) -> Endpoint<Req, Resp>
where
    S: PharmacyService,
    F: Fn(Arc<S>, Req) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Resp, Error>> + Send + 'static,
{
    Arc::new(move |req| Box::pin(f(Arc::clone(&svc), req)))
}

pub fn make_create_endpoint<S: PharmacyService>(
    svc: Arc<S>,
) -> Endpoint<CreateRequest, CreateResponse> {
    endpoint(svc, |svc, req: CreateRequest| async move {
        let id = svc.create(req.medicine).await?;
        Ok(CreateResponse { id })
    })
}

pub fn make_get_by_id_endpoint<S: PharmacyService>(
    svc: Arc<S>,
) -> Endpoint<GetByIdRequest, GetByIdResponse> {
    endpoint(svc, |svc, req: GetByIdRequest| async move {
        let medicine = svc.get_by_id(&req.id).await?;
        Ok(GetByIdResponse { medicine })
    })
}

pub fn make_sell_endpoint<S: PharmacyService>(svc: Arc<S>) -> Endpoint<SellRequest, SellResponse> {
    endpoint(svc, |svc, req: SellRequest| async move {
        svc.sell(&req.id, req.count).await?;
        Ok(SellResponse)
    })
}

/// Набор всех эндпоинтов сервиса — аналог структуры `Endpoints` в Go kit.
/// Клонирование дешёвое: копируются только указатели `Arc`.
#[derive(Clone)]
pub struct Endpoints {
    pub create: Endpoint<CreateRequest, CreateResponse>,
    pub get_by_id: Endpoint<GetByIdRequest, GetByIdResponse>,
    pub sell: Endpoint<SellRequest, SellResponse>,
}

impl Endpoints {
    pub fn new<S: PharmacyService>(svc: S) -> Self {
        let svc = Arc::new(svc);
        Self {
            create: make_create_endpoint(Arc::clone(&svc)),
            get_by_id: make_get_by_id_endpoint(Arc::clone(&svc)),
            sell: make_sell_endpoint(svc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pharmacy::{InMemoryRepository, Pharmacy};

    #[tokio::test]
    async fn endpoints_call_the_service() {
        let e = Endpoints::new(Pharmacy::new(InMemoryRepository::default()));
        let CreateResponse { id } = (e.create)(CreateRequest {
            medicine: Medicine {
                id: String::new(),
                name: "Нурофен".into(),
                manufacturer: "Reckitt".into(),
                price: 310.0,
                prescription: false,
                quantity: 3,
            },
        })
        .await
        .unwrap();

        (e.sell)(SellRequest {
            id: id.clone(),
            count: 2,
        })
        .await
        .unwrap();
        let resp = (e.get_by_id)(GetByIdRequest { id }).await.unwrap();
        assert_eq!(resp.medicine.quantity, 1);
    }
}
