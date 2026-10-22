# Многоэтапная сборка сервисов ЛР 6. Контекст сборки — корень воркспейса,
# так как crate lb6 зависит от Cargo.lock и workspace-hack:
#
#   docker build -f lb6/Dockerfile -t pharmacy-service .
#   docker build -f lb6/Dockerfile --build-arg BIN=orders -t order-service:v1 .
#
# Зависимости кэшируются отдельным слоем с помощью cargo-chef: при изменении
# только исходников пересобирается лишь код воркспейса. Файл не использует
# возможностей, специфичных для BuildKit, и собирается любым builder'ом.

ARG RUST_IMAGE=rust:1.99-alpine3.24
ARG RUNTIME_IMAGE=alpine:3.24

# ---------- Этап 0: инструменты сборки ----------
FROM ${RUST_IMAGE} AS chef
RUN apk add --no-cache musl-dev \
 && cargo install cargo-chef --locked --version 0.1.78
WORKDIR /src

# ---------- Этап 1: «рецепт» зависимостей из Cargo.toml / Cargo.lock ----------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---------- Этап 2: сборка статического бинарника (musl) ----------
FROM chef AS builder
COPY --from=planner /src/recipe.json recipe.json
# Этот слой инвалидируется только при изменении зависимостей.
RUN cargo chef cook --release --locked -p lb6 --recipe-path recipe.json
COPY . .
# ARG объявлен после cook: значение build-arg входит в ключ кэша следующих
# RUN, и при сборке другого бинарника зависимости не пересобираются.
ARG BIN=pharmacy
RUN cargo build --release --locked -p lb6 --bin "${BIN}" \
 && cp "target/release/${BIN}" /usr/local/bin/app

# ---------- Этап 3: минимальный образ для запуска ----------
FROM ${RUNTIME_IMAGE}
ARG BIN=pharmacy
LABEL org.opencontainers.image.title="${BIN}" \
      org.opencontainers.image.description="ЛР 6: микросервис на Rust" \
      org.opencontainers.image.authors="Цароев Альберт Казебкович"
RUN addgroup -S app && adduser -S -G app app
COPY --from=builder /usr/local/bin/app /usr/local/bin/app
USER app
# HTTP (оба сервиса) и gRPC (только pharmacy)
EXPOSE 8080 9090
ENTRYPOINT ["/usr/local/bin/app"]
