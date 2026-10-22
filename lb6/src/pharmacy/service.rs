//! Сервисный слой: интерфейс, реализация и middleware логирования.

use std::fmt::Debug;
use std::time::Instant;

use super::error::Error;
use super::model::Medicine;
use super::repository::MedicineRepository;

/// Интерфейс сервиса — аналог `type Service interface` из Go kit.
///
/// Параметр `ctx` из Go-сигнатур не нужен: отмена в Rust выражается
/// удалением (drop) future, а дедлайны — обёрткой `tokio::time::timeout`.
/// Возвращаемые future явно помечены `Send`, чтобы их можно было выполнять
/// на многопоточном рантайме tokio.
pub trait PharmacyService: Send + Sync + 'static {
    /// Добавляет лекарство и возвращает его идентификатор.
    fn create(&self, medicine: Medicine) -> impl Future<Output = Result<String, Error>> + Send;

    /// Возвращает лекарство по идентификатору.
    fn get_by_id(&self, id: &str) -> impl Future<Output = Result<Medicine, Error>> + Send;

    /// Продаёт `count` единиц лекарства.
    fn sell(&self, id: &str, count: u32) -> impl Future<Output = Result<(), Error>> + Send;
}

/// Реализация сервиса поверх произвольного репозитория.
#[derive(Debug, Default)]
pub struct Pharmacy<R> {
    repo: R,
}

impl<R: MedicineRepository> Pharmacy<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

fn validate(m: &Medicine) -> Result<(), Error> {
    if m.name.trim().is_empty() {
        return Err(Error::InvalidArgument("name не должно быть пустым".into()));
    }
    if m.manufacturer.trim().is_empty() {
        return Err(Error::InvalidArgument(
            "manufacturer не должно быть пустым".into(),
        ));
    }
    if !(m.price.is_finite() && m.price > 0.0) {
        return Err(Error::InvalidArgument(
            "price должна быть положительным числом".into(),
        ));
    }
    Ok(())
}

impl<R: MedicineRepository> PharmacyService for Pharmacy<R> {
    async fn create(&self, mut medicine: Medicine) -> Result<String, Error> {
        validate(&medicine)?;
        if medicine.id.trim().is_empty() {
            medicine.id = uuid::Uuid::new_v4().to_string();
        }
        let id = medicine.id.clone();
        self.repo.insert(medicine)?;
        Ok(id)
    }

    async fn get_by_id(&self, id: &str) -> Result<Medicine, Error> {
        self.repo.get(id)
    }

    async fn sell(&self, id: &str, count: u32) -> Result<(), Error> {
        if count == 0 {
            return Err(Error::InvalidArgument(
                "count должно быть больше нуля".into(),
            ));
        }
        self.repo.take(id, count).map(drop)
    }
}

/// Middleware логирования — аналог `loggingMiddleware` из примеров Go kit:
/// оборачивает любой сервис и пишет метод, аргументы, длительность и ошибку.
#[derive(Debug)]
pub struct Logging<S> {
    inner: S,
}

impl<S: PharmacyService> Logging<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

fn log_call<T>(method: &str, args: impl Debug, started: Instant, result: &Result<T, Error>) {
    let took = started.elapsed();
    match result {
        Ok(_) => tracing::info!(method, ?args, ?took, "ok"),
        Err(err) => tracing::warn!(method, ?args, ?took, %err, "error"),
    }
}

impl<S: PharmacyService> PharmacyService for Logging<S> {
    async fn create(&self, medicine: Medicine) -> Result<String, Error> {
        let started = Instant::now();
        let name = medicine.name.clone();
        let result = self.inner.create(medicine).await;
        log_call("create", name, started, &result);
        result
    }

    async fn get_by_id(&self, id: &str) -> Result<Medicine, Error> {
        let started = Instant::now();
        let result = self.inner.get_by_id(id).await;
        log_call("get_by_id", id, started, &result);
        result
    }

    async fn sell(&self, id: &str, count: u32) -> Result<(), Error> {
        let started = Instant::now();
        let result = self.inner.sell(id, count).await;
        log_call("sell", (id, count), started, &result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pharmacy::InMemoryRepository;

    fn svc() -> Pharmacy<InMemoryRepository> {
        Pharmacy::new(InMemoryRepository::default())
    }

    fn medicine() -> Medicine {
        Medicine {
            id: String::new(),
            name: "Амоксициллин".into(),
            manufacturer: "Sandoz".into(),
            price: 245.0,
            prescription: true,
            quantity: 20,
        }
    }

    #[tokio::test]
    async fn create_assigns_uuid_and_get_returns_it() {
        let svc = svc();
        let id = svc.create(medicine()).await.unwrap();
        assert!(uuid::Uuid::parse_str(&id).is_ok());
        let got = svc.get_by_id(&id).await.unwrap();
        assert_eq!(got.id, id);
        assert_eq!(got.name, "Амоксициллин");
    }

    #[tokio::test]
    async fn create_keeps_client_id_and_rejects_duplicates() {
        let svc = svc();
        let m = Medicine {
            id: "amx-500".into(),
            ..medicine()
        };
        assert_eq!(svc.create(m.clone()).await, Ok("amx-500".into()));
        assert_eq!(
            svc.create(m).await,
            Err(Error::AlreadyExists("amx-500".into()))
        );
    }

    #[tokio::test]
    async fn create_validates_input() {
        let svc = svc();
        for bad in [
            Medicine {
                name: " ".into(),
                ..medicine()
            },
            Medicine {
                manufacturer: String::new(),
                ..medicine()
            },
            Medicine {
                price: 0.0,
                ..medicine()
            },
            Medicine {
                price: f64::NAN,
                ..medicine()
            },
        ] {
            assert!(matches!(
                svc.create(bad).await,
                Err(Error::InvalidArgument(_))
            ));
        }
    }

    #[tokio::test]
    async fn sell_decrements_and_checks_stock() {
        let svc = Logging::new(svc());
        let id = svc.create(medicine()).await.unwrap();
        svc.sell(&id, 15).await.unwrap();
        assert_eq!(svc.get_by_id(&id).await.unwrap().quantity, 5);
        assert!(matches!(
            svc.sell(&id, 6).await,
            Err(Error::InsufficientStock {
                available: 5,
                requested: 6,
                ..
            })
        ));
        assert!(matches!(
            svc.sell(&id, 0).await,
            Err(Error::InvalidArgument(_))
        ));
        assert!(matches!(
            svc.sell("missing", 1).await,
            Err(Error::NotFound(_))
        ));
    }
}
