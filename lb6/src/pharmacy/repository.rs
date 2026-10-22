//! Слой хранения.

use std::collections::HashMap;
use std::sync::{PoisonError, RwLock};

use super::error::Error;
use super::model::Medicine;

/// Хранилище лекарств. Методы синхронные: in-memory операции не ждут
/// ввода-вывода, а блокировка удерживается микросекунды.
pub trait MedicineRepository: Send + Sync + 'static {
    /// Сохраняет новое лекарство.
    ///
    /// # Errors
    /// [`Error::AlreadyExists`], если id занят.
    fn insert(&self, medicine: Medicine) -> Result<(), Error>;

    /// Возвращает копию лекарства.
    ///
    /// # Errors
    /// [`Error::NotFound`], если id неизвестен.
    fn get(&self, id: &str) -> Result<Medicine, Error>;

    /// Атомарно уменьшает остаток на `count` и возвращает новый остаток.
    /// Проверка и списание выполняются под одной блокировкой, поэтому
    /// параллельные продажи не уводят остаток в минус.
    ///
    /// # Errors
    /// [`Error::NotFound`] или [`Error::InsufficientStock`].
    fn take(&self, id: &str, count: u32) -> Result<u32, Error>;
}

/// Потокобезопасное хранилище в памяти процесса.
#[derive(Debug, Default)]
pub struct InMemoryRepository {
    items: RwLock<HashMap<String, Medicine>>,
}

impl MedicineRepository for InMemoryRepository {
    fn insert(&self, medicine: Medicine) -> Result<(), Error> {
        // Отравление блокировки возможно только при панике внутри
        // критической секции; данные при этом согласованы, берём их как есть.
        let mut items = self.items.write().unwrap_or_else(PoisonError::into_inner);
        if items.contains_key(&medicine.id) {
            return Err(Error::AlreadyExists(medicine.id));
        }
        items.insert(medicine.id.clone(), medicine);
        Ok(())
    }

    fn get(&self, id: &str) -> Result<Medicine, Error> {
        let items = self.items.read().unwrap_or_else(PoisonError::into_inner);
        items
            .get(id)
            .cloned()
            .ok_or_else(|| Error::NotFound(id.to_owned()))
    }

    fn take(&self, id: &str, count: u32) -> Result<u32, Error> {
        let mut items = self.items.write().unwrap_or_else(PoisonError::into_inner);
        let medicine = items
            .get_mut(id)
            .ok_or_else(|| Error::NotFound(id.to_owned()))?;
        medicine.quantity =
            medicine
                .quantity
                .checked_sub(count)
                .ok_or_else(|| Error::InsufficientStock {
                    id: id.to_owned(),
                    available: medicine.quantity,
                    requested: count,
                })?;
        Ok(medicine.quantity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    fn aspirin(quantity: u32) -> Medicine {
        Medicine {
            id: "m1".into(),
            name: "Аспирин".into(),
            manufacturer: "Bayer".into(),
            price: 120.5,
            prescription: false,
            quantity,
        }
    }

    #[test]
    fn insert_get_take() {
        let repo = InMemoryRepository::default();
        repo.insert(aspirin(10)).unwrap();
        assert_eq!(repo.get("m1").unwrap().quantity, 10);
        assert_eq!(repo.take("m1", 4), Ok(6));
        assert_eq!(repo.get("m1").unwrap().quantity, 6);
    }

    #[test]
    fn errors() {
        let repo = InMemoryRepository::default();
        repo.insert(aspirin(1)).unwrap();
        assert_eq!(
            repo.insert(aspirin(1)),
            Err(Error::AlreadyExists("m1".into()))
        );
        assert_eq!(repo.get("nope"), Err(Error::NotFound("nope".into())));
        assert_eq!(
            repo.take("m1", 2),
            Err(Error::InsufficientStock {
                id: "m1".into(),
                available: 1,
                requested: 2
            })
        );
        // Неудачная продажа не меняет остаток.
        assert_eq!(repo.get("m1").unwrap().quantity, 1);
    }

    #[test]
    fn concurrent_takes_never_oversell() {
        let repo = Arc::new(InMemoryRepository::default());
        repo.insert(aspirin(50)).unwrap();
        let handles: Vec<_> = (0..100)
            .map(|_| {
                let repo = Arc::clone(&repo);
                thread::spawn(move || repo.take("m1", 1).is_ok())
            })
            .collect();
        let sold = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .filter(|&ok| ok)
            .count();
        assert_eq!(sold, 50);
        assert_eq!(repo.get("m1").unwrap().quantity, 0);
    }
}
