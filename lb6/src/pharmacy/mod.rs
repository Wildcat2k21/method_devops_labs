//! Сервис «Аптека» (вариант 10).
//!
//! Слои повторяют архитектуру Go kit:
//!
//! | Go kit                          | Этот модуль                          |
//! |---------------------------------|--------------------------------------|
//! | `type Service interface`        | трейт [`service::PharmacyService`]   |
//! | реализация сервиса              | [`service::Pharmacy`]                |
//! | service middleware (logging)    | [`service::Logging`]                 |
//! | `endpoint.Endpoint`             | [`endpoint::Endpoint`]               |
//! | `transport/http`                | [`transport::http`] (axum)           |
//! | `transport/grpc`                | [`transport::grpc`] (tonic)          |
//! | in-memory repository            | [`repository::InMemoryRepository`]   |

pub mod endpoint;
pub mod error;
pub mod model;
pub mod repository;
pub mod service;
pub mod transport;

pub use endpoint::Endpoints;
pub use error::Error;
pub use model::Medicine;
pub use repository::{InMemoryRepository, MedicineRepository};
pub use service::{Logging, Pharmacy, PharmacyService};
