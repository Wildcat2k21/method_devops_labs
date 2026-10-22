//! Задание 1 — базовый HTTP-сервис заказов.
//!
//! Повторяет пример из методички на «голом» `std::net::TcpListener` по мотивам
//! главы 21.1 Rust Book («Building a Single-Threaded Web Server»), до
//! появления пула потоков. Сервер однопоточный: соединения обрабатываются
//! по очереди, поэтому хранилище передаётся как `&mut OrderStore` и не
//! требует синхронизации.
//!
//! Эндпоинты:
//! * `POST /orders`      — создать заказ;
//! * `GET  /orders/{id}` — получить заказ по идентификатору;
//! * `GET  /orders`      — список всех заказов.

pub mod http;
pub mod router;
pub mod store;

use std::io;
use std::net::TcpListener;
use std::time::Duration;

pub use store::{NewOrder, Order, OrderStore};

/// Сколько ждать данных от клиента, прежде чем закрыть соединение. Без
/// тайм-аута один «молчащий» клиент заблокировал бы однопоточный сервер.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Обрабатывает входящие соединения до тех пор, пока работает `listener`.
pub fn serve(listener: &TcpListener, store: &mut OrderStore) -> io::Result<()> {
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ошибка accept: {e}");
                continue;
            }
        };
        stream.set_read_timeout(Some(READ_TIMEOUT))?;

        let response = match http::Request::read_from(&mut stream) {
            Ok(request) => {
                let response = router::handle(&request, store);
                eprintln!(
                    "{} {} -> {}",
                    request.method, request.path, response.status.code
                );
                response
            }
            Err(e) => http::Response::error(http::Status::BAD_REQUEST, &e.to_string()),
        };
        if let Err(e) = response.write_to(&mut stream) {
            eprintln!("ошибка записи ответа: {e}");
        }
    }
    Ok(())
}
