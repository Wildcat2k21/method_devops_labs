//! Задание 1: `cargo run -p lb6 --bin orders`, затем
//! `curl localhost:8080/orders/1`.

use std::net::TcpListener;

use lb6::orders::{self, OrderStore};

fn main() -> std::io::Result<()> {
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_owned());
    let listener = TcpListener::bind(&addr)?;
    eprintln!("order-service слушает http://{addr}");

    let mut store = OrderStore::new();
    orders::serve(&listener, &mut store)
}
