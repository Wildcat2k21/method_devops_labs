//! Маршрутизация запросов к хранилищу заказов.

use super::http::{Request, Response, Status};
use super::store::{NewOrder, OrderStore};

/// Обрабатывает запрос и формирует ответ. Функция не зависит от сокетов,
/// поэтому легко тестируется.
pub fn handle(req: &Request, store: &mut OrderStore) -> Response {
    let segments: Vec<&str> = req.path.trim_matches('/').split('/').collect();
    match (req.method.as_str(), segments.as_slice()) {
        ("POST", ["orders"]) => create(req, store),
        ("GET", ["orders"]) => Response::json(Status::OK, &store.list()),
        ("GET", ["orders", id]) => match id.parse::<u64>() {
            Ok(id) => match store.get(id) {
                Some(order) => Response::json(Status::OK, order),
                None => Response::error(Status::NOT_FOUND, &format!("заказ {id} не найден")),
            },
            Err(_) => Response::error(
                Status::BAD_REQUEST,
                "идентификатор заказа должен быть числом",
            ),
        },
        (_, ["orders"] | ["orders", _]) => {
            Response::error(Status::METHOD_NOT_ALLOWED, "метод не поддерживается")
        }
        _ => Response::error(Status::NOT_FOUND, "маршрут не найден"),
    }
}

fn create(req: &Request, store: &mut OrderStore) -> Response {
    let new: NewOrder = match serde_json::from_slice(&req.body) {
        Ok(new) => new,
        Err(e) => {
            return Response::error(Status::BAD_REQUEST, &format!("некорректный JSON: {e}"));
        }
    };
    if new.customer.trim().is_empty() || new.item.trim().is_empty() {
        return Response::error(
            Status::BAD_REQUEST,
            "customer и item не должны быть пустыми",
        );
    }
    if new.quantity == 0 {
        return Response::error(Status::BAD_REQUEST, "quantity должно быть больше нуля");
    }
    Response::json(Status::CREATED, &store.create(new))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn request(method: &str, path: &str, body: &str) -> Request {
        Request {
            method: method.to_owned(),
            path: path.to_owned(),
            body: body.as_bytes().to_vec(),
        }
    }

    fn body(resp: &Response) -> Value {
        serde_json::from_str(&resp.body).unwrap()
    }

    #[test]
    fn create_get_list_roundtrip() {
        let mut store = OrderStore::new();
        let resp = handle(
            &request(
                "POST",
                "/orders",
                r#"{"customer":"Иван","item":"Ноутбук","quantity":1}"#,
            ),
            &mut store,
        );
        assert_eq!(resp.status, Status::CREATED);
        assert_eq!(body(&resp)["id"], 1);

        let resp = handle(&request("GET", "/orders/1", ""), &mut store);
        assert_eq!(resp.status, Status::OK);
        assert_eq!(
            body(&resp),
            json!({"id":1,"customer":"Иван","item":"Ноутбук","quantity":1})
        );

        handle(
            &request(
                "POST",
                "/orders/",
                r#"{"customer":"Анна","item":"Мышь","quantity":2}"#,
            ),
            &mut store,
        );
        let resp = handle(&request("GET", "/orders", ""), &mut store);
        assert_eq!(body(&resp).as_array().unwrap().len(), 2);
    }

    #[test]
    fn error_responses() {
        let mut store = OrderStore::new();
        let cases = [
            (request("GET", "/orders/42", ""), Status::NOT_FOUND),
            (request("GET", "/orders/abc", ""), Status::BAD_REQUEST),
            (request("POST", "/orders", "not json"), Status::BAD_REQUEST),
            (
                request(
                    "POST",
                    "/orders",
                    r#"{"customer":"","item":"x","quantity":1}"#,
                ),
                Status::BAD_REQUEST,
            ),
            (
                request(
                    "POST",
                    "/orders",
                    r#"{"customer":"a","item":"x","quantity":0}"#,
                ),
                Status::BAD_REQUEST,
            ),
            (
                request("DELETE", "/orders/1", ""),
                Status::METHOD_NOT_ALLOWED,
            ),
            (request("GET", "/unknown", ""), Status::NOT_FOUND),
        ];
        for (req, expected) in cases {
            let resp = handle(&req, &mut store);
            assert_eq!(resp.status, expected, "{} {}", req.method, req.path);
            assert!(body(&resp)["error"].is_string());
        }
    }
}
