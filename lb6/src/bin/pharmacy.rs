//! Задания 2–3: сервис «Аптека», HTTP на :8080 и gRPC на :9090 одновременно.
//!
//! Адреса переопределяются переменными окружения `HTTP_ADDR` и `GRPC_ADDR`,
//! уровень логирования — `RUST_LOG` (по умолчанию `info`).

use std::error::Error;
use std::io::IsTerminal;

use lb6::pharmacy::transport::{grpc, http};
use lb6::pharmacy::{Endpoints, InMemoryRepository, Logging, Pharmacy};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tonic::transport::server::TcpIncoming;
use tracing_subscriber::EnvFilter;

type BoxError = Box<dyn Error + Send + Sync>;

/// Ждёт Ctrl+C или SIGTERM (его посылает `docker stop`).
async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("обработчик SIGTERM");
        tokio::select! {
            _ = ctrl_c => {},
            _ = term.recv() => {},
        }
    }
    #[cfg(not(unix))]
    ctrl_c.await.ok();
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        // Цвета только в терминале: в `docker logs` escape-коды мешают.
        .with_ansi(std::io::stdout().is_terminal())
        .init();

    let http_addr = std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let grpc_addr = std::env::var("GRPC_ADDR").unwrap_or_else(|_| "0.0.0.0:9090".into());

    // Сборка слоёв: репозиторий -> сервис -> middleware -> эндпоинты.
    let service = Logging::new(Pharmacy::new(InMemoryRepository::default()));
    let endpoints = Endpoints::new(service);

    // Один сигнал завершения на оба транспорта.
    let (stop_tx, stop_rx) = watch::channel(());
    tokio::spawn(async move {
        shutdown_signal().await;
        tracing::info!("получен сигнал завершения, останавливаем серверы");
        stop_tx.send(()).ok();
    });
    let stopped = |mut rx: watch::Receiver<()>| async move {
        rx.changed().await.ok();
    };

    let http_listener = TcpListener::bind(&http_addr).await?;
    let grpc_listener = TcpListener::bind(&grpc_addr).await?;
    tracing::info!("HTTP-транспорт слушает http://{http_addr}");
    tracing::info!("gRPC-транспорт слушает {grpc_addr}");

    let http_server = axum::serve(http_listener, http::router(endpoints.clone()))
        .with_graceful_shutdown(stopped(stop_rx.clone()));
    let grpc_server = tonic::transport::Server::builder()
        .add_service(grpc::server(endpoints))
        .add_service(grpc::reflection_server())
        .serve_with_incoming_shutdown(TcpIncoming::from(grpc_listener), stopped(stop_rx));

    let http = async { http_server.await.map_err(BoxError::from) };
    let grpc = async { grpc_server.await.map_err(BoxError::from) };
    tokio::try_join!(http, grpc)?;
    tracing::info!("серверы остановлены");
    Ok(())
}
