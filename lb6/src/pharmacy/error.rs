//! Ошибки бизнес-логики. Транспорты отображают их в коды HTTP и gRPC.

/// Ошибка сервиса «Аптека».
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("лекарство {0} не найдено")]
    NotFound(String),
    #[error("лекарство с id {0} уже существует")]
    AlreadyExists(String),
    #[error("некорректные данные: {0}")]
    InvalidArgument(String),
    #[error("недостаточно товара {id}: в наличии {available}, запрошено {requested}")]
    InsufficientStock {
        id: String,
        available: u32,
        requested: u32,
    },
}
