//! Доменная модель.

use serde::{Deserialize, Serialize};

/// Лекарство.
///
/// Поля соответствуют варианту 10: `ID, Name, Manufacturer, Price float64,
/// Prescription bool, Quantity int`. Количество — `u32`, так как
/// отрицательный остаток не имеет смысла и отсекается уже на уровне типов.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Medicine {
    /// Пустой при создании — тогда идентификатор назначает сервис.
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub price: f64,
    /// Отпускается только по рецепту.
    #[serde(default)]
    pub prescription: bool,
    pub quantity: u32,
}
