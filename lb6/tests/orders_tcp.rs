//! Задание 1: сервер заказов через настоящий TCP-сокет.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

use lb6::orders::{self, OrderStore};
use serde_json::Value;

fn start_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        let mut store = OrderStore::new();
        orders::serve(&listener, &mut store).unwrap();
    });
    addr
}

fn send(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, Value) {
    let mut stream = TcpStream::connect(addr).unwrap();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut raw = String::new();
    stream.read_to_string(&mut raw).unwrap();
    let (head, body) = raw.split_once("\r\n\r\n").unwrap();
    let status = head.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, serde_json::from_str(body).unwrap())
}

#[test]
fn data_persists_between_requests() {
    let addr = start_server();

    let (status, created) = send(
        addr,
        "POST",
        "/orders",
        r#"{"customer":"Альберт","item":"Клавиатура","quantity":2}"#,
    );
    assert_eq!(status, 201);
    assert_eq!(created["id"], 1);

    let (status, order) = send(addr, "GET", "/orders/1", "");
    assert_eq!(status, 200);
    assert_eq!(order["item"], "Клавиатура");

    send(
        addr,
        "POST",
        "/orders",
        r#"{"customer":"Мария","item":"Монитор","quantity":1}"#,
    );
    let (status, list) = send(addr, "GET", "/orders", "");
    assert_eq!(status, 200);
    assert_eq!(list.as_array().unwrap().len(), 2);

    let (status, err) = send(addr, "GET", "/orders/99", "");
    assert_eq!(status, 404);
    assert!(err["error"].as_str().unwrap().contains("99"));
}
