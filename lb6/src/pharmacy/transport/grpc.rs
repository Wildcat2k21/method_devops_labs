//! gRPC-транспорт на tonic — аналог `github.com/go-kit/kit/transport/grpc`.
//!
//! Каждый метод: декодирует protobuf-сообщение в запрос эндпоинта,
//! вызывает эндпоинт и кодирует ответ обратно; ошибки сервиса
//! отображаются в коды gRPC.

use tonic::{Request, Response, Status};
use tonic_reflection::server::v1::{ServerReflection, ServerReflectionServer};

use crate::pharmacy::endpoint::{CreateRequest, Endpoints, GetByIdRequest, SellRequest};
use crate::pharmacy::{Error, Medicine};

/// Код, сгенерированный из `proto/pharmacy.proto` в `build.rs`.
#[allow(clippy::all, clippy::pedantic)]
pub mod pb {
    tonic::include_proto!("pharmacy.v1");

    /// Дескриптор `.proto` для gRPC reflection.
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("pharmacy_descriptor");
}

use pb::pharmacy_server::{Pharmacy, PharmacyServer};

/// Реализация сгенерированного трейта `Pharmacy` поверх эндпоинтов.
pub struct GrpcServer {
    endpoints: Endpoints,
}

/// gRPC-сервис, готовый к добавлению в `tonic::transport::Server`.
pub fn server(endpoints: Endpoints) -> PharmacyServer<GrpcServer> {
    PharmacyServer::new(GrpcServer { endpoints })
}

/// Сервис reflection: позволяет `grpcurl` обходиться без `.proto`-файла.
///
/// # Panics
/// Если встроенный дескриптор повреждён (ошибка сборки, а не времени работы).
pub fn reflection_server() -> ServerReflectionServer<impl ServerReflection> {
    tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pb::FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("встроенный дескриптор protobuf корректен")
}

impl From<pb::Medicine> for Medicine {
    fn from(m: pb::Medicine) -> Self {
        Self {
            id: m.id,
            name: m.name,
            manufacturer: m.manufacturer,
            price: m.price,
            prescription: m.prescription,
            quantity: m.quantity,
        }
    }
}

impl From<Medicine> for pb::Medicine {
    fn from(m: Medicine) -> Self {
        Self {
            id: m.id,
            name: m.name,
            manufacturer: m.manufacturer,
            price: m.price,
            prescription: m.prescription,
            quantity: m.quantity,
        }
    }
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        let msg = err.to_string();
        match err {
            Error::NotFound(_) => Status::not_found(msg),
            Error::AlreadyExists(_) => Status::already_exists(msg),
            Error::InvalidArgument(_) => Status::invalid_argument(msg),
            Error::InsufficientStock { .. } => Status::failed_precondition(msg),
        }
    }
}

#[tonic::async_trait]
impl Pharmacy for GrpcServer {
    async fn create(
        &self,
        request: Request<pb::CreateRequest>,
    ) -> Result<Response<pb::CreateResponse>, Status> {
        let medicine = request
            .into_inner()
            .medicine
            .ok_or_else(|| Status::invalid_argument("поле medicine обязательно"))?;
        let resp = (self.endpoints.create)(CreateRequest {
            medicine: medicine.into(),
        })
        .await?;
        Ok(Response::new(pb::CreateResponse { id: resp.id }))
    }

    async fn get_by_id(
        &self,
        request: Request<pb::GetByIdRequest>,
    ) -> Result<Response<pb::GetByIdResponse>, Status> {
        let pb::GetByIdRequest { id } = request.into_inner();
        let resp = (self.endpoints.get_by_id)(GetByIdRequest { id }).await?;
        Ok(Response::new(pb::GetByIdResponse {
            medicine: Some(resp.medicine.into()),
        }))
    }

    async fn sell(
        &self,
        request: Request<pb::SellRequest>,
    ) -> Result<Response<pb::SellResponse>, Status> {
        let pb::SellRequest { id, count } = request.into_inner();
        (self.endpoints.sell)(SellRequest { id, count }).await?;
        Ok(Response::new(pb::SellResponse {}))
    }
}
