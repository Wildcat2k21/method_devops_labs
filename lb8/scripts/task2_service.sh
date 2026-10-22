#!/usr/bin/env bash
# ЛР 8, задание 2: образ сервиса «Аптека» из ЛР 6, запуск контейнера и
# проверка эндпоинтов. Запускать из корня воркспейса.
set -euo pipefail
source "$(dirname "$0")/lib.sh"

run docker build -f lb6/Dockerfile -t pharmacy-service:v1 .
run docker images pharmacy-service

run docker run -d --name pharmacy -p 8080:8080 -p 9090:9090 pharmacy-service:v1
wait_for 40 curl -fs http://localhost:8080/health
run docker ps --filter name=pharmacy

run curl -s http://localhost:8080/health
run curl -s -X POST http://localhost:8080/medicines -H 'Content-Type: application/json' \
    -d '{"id":"asp-500","name":"Аспирин","manufacturer":"Bayer","price":120.5,"prescription":false,"quantity":30}'
run curl -s -i -X POST http://localhost:8080/medicines/asp-500/sell -H 'Content-Type: application/json' -d '{"count":12}'
run curl -s http://localhost:8080/medicines/asp-500
run curl -s -i http://localhost:8080/medicines/unknown
run grpcurl -plaintext -d '{"id":"asp-500"}' localhost:9090 pharmacy.v1.Pharmacy/GetByID

run docker exec pharmacy id
run docker stop pharmacy
run docker logs pharmacy
run docker rm pharmacy
