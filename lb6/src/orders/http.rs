//! Минимальный разбор HTTP/1.1-запросов и формирование ответов.

use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};

/// Максимальный размер тела запроса.
const MAX_BODY: usize = 1 << 20;
/// Максимальная длина строки запроса или заголовка.
const MAX_LINE: usize = 8 * 1024;

/// Разобранный HTTP-запрос.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub body: Vec<u8>,
}

/// Ошибка разбора запроса.
#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Malformed(&'static str),
    BodyTooLarge(usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "ошибка чтения запроса: {e}"),
            Self::Malformed(what) => write!(f, "некорректный запрос: {what}"),
            Self::BodyTooLarge(n) => write!(f, "тело запроса слишком большое: {n} байт"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

fn read_line<R: BufRead>(reader: &mut R) -> Result<String, ParseError> {
    let mut line = String::new();
    let n = reader.by_ref().take(MAX_LINE as u64).read_line(&mut line)?;
    if n == 0 {
        return Err(ParseError::Malformed(
            "соединение закрыто до конца заголовков",
        ));
    }
    if !line.ends_with('\n') {
        return Err(ParseError::Malformed("слишком длинная строка"));
    }
    Ok(line.trim_end_matches(['\r', '\n']).to_owned())
}

impl Request {
    /// Читает запрос: строку запроса, заголовки и тело длиной `Content-Length`.
    pub fn read_from<R: Read>(stream: R) -> Result<Self, ParseError> {
        let mut reader = BufReader::new(stream);

        let request_line = read_line(&mut reader)?;
        let mut parts = request_line.split_whitespace();
        let (Some(method), Some(target), Some(version)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return Err(ParseError::Malformed("строка запроса"));
        };
        if !version.starts_with("HTTP/1.") {
            return Err(ParseError::Malformed("поддерживается только HTTP/1.x"));
        }

        let mut content_length = 0_usize;
        loop {
            let line = read_line(&mut reader)?;
            if line.is_empty() {
                break;
            }
            let Some((name, value)) = line.split_once(':') else {
                return Err(ParseError::Malformed("заголовок"));
            };
            if name.trim().eq_ignore_ascii_case("content-length") {
                content_length = value
                    .trim()
                    .parse()
                    .map_err(|_| ParseError::Malformed("Content-Length"))?;
            }
        }
        if content_length > MAX_BODY {
            return Err(ParseError::BodyTooLarge(content_length));
        }

        let mut body = vec![0; content_length];
        reader.read_exact(&mut body)?;

        // Строку запроса (?a=b) сервис не использует — отбрасываем.
        let path = target.split('?').next().unwrap_or(target).to_owned();
        Ok(Self {
            method: method.to_owned(),
            path,
            body,
        })
    }
}

/// Код и текст статуса ответа.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    pub code: u16,
    pub reason: &'static str,
}

impl Status {
    pub const OK: Self = Self::new(200, "OK");
    pub const CREATED: Self = Self::new(201, "Created");
    pub const BAD_REQUEST: Self = Self::new(400, "Bad Request");
    pub const NOT_FOUND: Self = Self::new(404, "Not Found");
    pub const METHOD_NOT_ALLOWED: Self = Self::new(405, "Method Not Allowed");

    const fn new(code: u16, reason: &'static str) -> Self {
        Self { code, reason }
    }
}

/// HTTP-ответ с JSON-телом.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: Status,
    pub body: String,
}

impl Response {
    /// Ответ с сериализованным в JSON значением.
    pub fn json<T: serde::Serialize>(status: Status, value: &T) -> Self {
        let body = serde_json::to_string(value).expect("сериализация в JSON не падает");
        Self { status, body }
    }

    /// Ответ вида `{"error": "..."}`.
    pub fn error(status: Status, message: &str) -> Self {
        Self::json(status, &serde_json::json!({ "error": message }))
    }

    pub fn write_to<W: Write>(&self, mut out: W) -> io::Result<()> {
        write!(
            out,
            "HTTP/1.1 {} {}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            self.status.code,
            self.status.reason,
            self.body.len(),
            self.body
        )?;
        out.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_get_without_body() {
        let raw = "GET /orders/1?verbose=1 HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = Request::read_from(raw.as_bytes()).unwrap();
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/orders/1");
        assert!(req.body.is_empty());
    }

    #[test]
    fn parses_post_with_body() {
        let raw = "POST /orders HTTP/1.1\r\ncontent-length: 7\r\n\r\n{\"a\":1}";
        let req = Request::read_from(raw.as_bytes()).unwrap();
        assert_eq!(req.body, br#"{"a":1}"#);
    }

    #[test]
    fn rejects_garbage() {
        assert!(Request::read_from("hello\r\n\r\n".as_bytes()).is_err());
        assert!(Request::read_from("GET / SPDY/3\r\n\r\n".as_bytes()).is_err());
        let huge = format!(
            "POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            MAX_BODY + 1
        );
        assert!(matches!(
            Request::read_from(huge.as_bytes()),
            Err(ParseError::BodyTooLarge(_))
        ));
    }

    #[test]
    fn writes_response() {
        let mut out = Vec::new();
        Response::error(Status::NOT_FOUND, "нет")
            .write_to(&mut out)
            .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("HTTP/1.1 404 Not Found\r\n"));
        assert!(text.ends_with(r#"{"error":"нет"}"#));
        // Content-Length считается в байтах, а не в символах.
        assert!(text.contains("Content-Length: 18\r\n"));
    }
}
