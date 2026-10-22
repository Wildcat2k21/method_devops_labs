//! In-memory хранилище заказов.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Заказ.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub customer: String,
    pub item: String,
    pub quantity: u32,
}

/// Тело запроса на создание заказа.
#[derive(Debug, Clone, Deserialize)]
pub struct NewOrder {
    pub customer: String,
    pub item: String,
    pub quantity: u32,
}

/// Хранилище заказов в памяти процесса. Идентификаторы выдаются
/// последовательно начиная с 1, поэтому первый заказ доступен по `/orders/1`.
#[derive(Debug, Default)]
pub struct OrderStore {
    orders: BTreeMap<u64, Order>,
    next_id: u64,
}

impl OrderStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, new: NewOrder) -> Order {
        self.next_id += 1;
        let order = Order {
            id: self.next_id,
            customer: new.customer,
            item: new.item,
            quantity: new.quantity,
        };
        self.orders.insert(order.id, order.clone());
        order
    }

    pub fn get(&self, id: u64) -> Option<&Order> {
        self.orders.get(&id)
    }

    /// Все заказы в порядке возрастания идентификатора.
    pub fn list(&self) -> Vec<&Order> {
        self.orders.values().collect()
    }
}
