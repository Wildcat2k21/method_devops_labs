# Контейнерные технологии и DevOps для ИТ-специалистов

[![CI](https://github.com/Wildcat2k21/method_devops_labs/actions/workflows/ci.yml/badge.svg)](https://github.com/Wildcat2k21/method_devops_labs/actions/workflows/ci.yml)
[![Docker](https://github.com/Wildcat2k21/method_devops_labs/actions/workflows/docker.yml/badge.svg)](https://github.com/Wildcat2k21/method_devops_labs/actions/workflows/docker.yml)

**Имя студента:** Цароев Альберт Казебкович\
**Группа:** ПИЖ-б-о-23-1(1)

Лабораторные работы выполнены на **Rust** (вместо Go) в едином
Cargo-воркспейсе.

| №  | Тема                                             | Каталог          | Отчёт                       |
|----|--------------------------------------------------|------------------|-----------------------------|
| 1  | Обработка одномерных и двумерных массивов        | [lb1](lb1)       | [README](lb1/README.md)     |
| 2  | Рекурсия, вставка и удаление элементов           | [lb2](lb2)       | [README](lb2/README.md)     |
| 3  | Очередь и стек                                   | [lb3](lb3)       | [README](lb3/README.md)     |
| 4  | Конкурентные вычисления (аналог горутин)         | [lb4](lb4)       | [README](lb4/README.md)     |
| 5  | Основы Linux и Git                               | [lb5](lb5)       | [README](lb5/README.md)     |
| 6  | Микросервисы: HTTP и gRPC (вариант 10, «Аптека») | [lb6](lb6)       | [README](lb6/README.md)     |
| 7  | CI/CD на GitHub Actions (вариант 10, уведомления)| [lb7](lb7)       | [README](lb7/README.md)     |
| 8  | Основы Docker (вариант 10, cron)                 | [lb8](lb8)       | [README](lb8/README.md)     |

## Структура

```text
.
├── Cargo.toml              — воркспейс: участники, общие версии зависимостей, линты
├── Cargo.lock
├── .config/hakari.toml     — настройки cargo-hakari
├── workspace-hack/         — крейт, унифицирующий features зависимостей (cargo-hakari)
├── lb1 … lb4, lb6/         — крейты лабораторных работ
├── lb5/, lb7/, lb8/        — работы без Rust-кода: скрипты, конфигурации, отчёты
├── .github/workflows/      — CI/CD (ЛР 7)
├── .dockerignore           — контекст сборки Docker-образов (ЛР 7, ЛР 8)
└── tools/
    ├── render_diagrams.py  — PlantUML (*.puml) → SVG через локальный PlantUML Server
    └── build_pdf.py        — README.md → PDF через PrinceXML
```

## Сборка и проверка

Требуется Rust 1.88 или новее.

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo hakari verify          # cargo install cargo-hakari
```

Запуск отдельных работ:

```bash
cargo run -p lb1
cargo run -p lb4 -- task1
cargo run -p lb6 --bin orders       # ЛР 6, задание 1 — :8080
cargo run -p lb6 --bin pharmacy     # ЛР 6, задания 2–3 — HTTP :8080, gRPC :9090
docker build -f lb6/Dockerfile -t pharmacy-service .
```

Диаграммы и PDF-версии отчётов:

```bash
docker run -d -p 127.0.0.1:8080:8080 plantuml/plantuml-server:jetty
python3 tools/render_diagrams.py     # все lb*/docs/*.puml → *.svg
python3 tools/build_pdf.py           # все lb*/README.md → lb*/README.pdf
```

## Почему один воркспейс и cargo-hakari

Все крейты используют общий `Cargo.lock` и общий каталог `target/`, а версии
зависимостей задаются один раз в `[workspace.dependencies]`. Крейт
`workspace-hack`, который генерирует `cargo hakari`, объединяет наборы
features общих зависимостей. Без него, например, `tokio` компилировался бы
заново с разными features при сборке `lb4` и `lb6` по отдельности. В CI
`cargo hakari verify` проверяет, что крейт не устарел.
