//! Задание 3: gRPC-транспорт — реальный сервер на случайном порту и
//! сгенерированный tonic-клиент.

use lb6::pharmacy::transport::grpc::{self, pb};
use lb6::pharmacy::{Endpoints, InMemoryRepository, Pharmacy};
use pb::pharmacy_client::PharmacyClient;
use tokio::net::TcpListener;
use tonic::Code;
use tonic::transport::{Channel, server::TcpIncoming};

async fn client() -> PharmacyClient<Channel> {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let endpoints = Endpoints::new(Pharmacy::new(InMemoryRepository::default()));
    tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(grpc::server(endpoints))
            .serve_with_incoming(TcpIncoming::from(listener)),
    );
    PharmacyClient::connect(format!("http://{addr}"))
        .await
        .unwrap()
}

fn insulin() -> pb::Medicine {
    pb::Medicine {
        id: String::new(),
        name: "Инсулин".into(),
        manufacturer: "Novo Nordisk".into(),
        price: 1290.0,
        prescription: true,
        quantity: 5,
    }
}

#[tokio::test]
async fn create_get_sell() {
    let mut c = client().await;

    let id = c
        .create(pb::CreateRequest {
            medicine: Some(insulin()),
        })
        .await
        .unwrap()
        .into_inner()
        .id;

    c.sell(pb::SellRequest {
        id: id.clone(),
        count: 2,
    })
    .await
    .unwrap();

    let m = c
        .get_by_id(pb::GetByIdRequest { id: id.clone() })
        .await
        .unwrap()
        .into_inner()
        .medicine
        .unwrap();
    assert_eq!(m.id, id);
    assert!(m.prescription);
    assert_eq!(m.quantity, 3);
}

#[tokio::test]
async fn errors_map_to_grpc_codes() {
    let mut c = client().await;

    let err = c
        .get_by_id(pb::GetByIdRequest { id: "nope".into() })
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);

    let err = c
        .create(pb::CreateRequest { medicine: None })
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);

    let id = c
        .create(pb::CreateRequest {
            medicine: Some(insulin()),
        })
        .await
        .unwrap()
        .into_inner()
        .id;
    let err = c.sell(pb::SellRequest { id, count: 6 }).await.unwrap_err();
    assert_eq!(err.code(), Code::FailedPrecondition);
}
