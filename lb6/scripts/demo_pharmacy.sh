#!/usr/bin/env bash
# ЛР 6, задания 2–3: проверка сервиса «Аптека» через HTTP (curl) и gRPC (grpcurl).
# Перед запуском: cargo run -p lb6 --bin pharmacy
set -euo pipefail

BASE=${BASE:-http://localhost:8080}
GRPC=${GRPC:-localhost:9090}
GRPCURL=${GRPCURL:-grpcurl}

# Печатает команду в виде, пригодном для копирования в терминал, и выполняет её.
run() {
    local line='$' arg
    for arg in "$@"; do
        if [[ $arg =~ [^A-Za-z0-9_./:=@%+,-] ]]; then
            line+=" '${arg//\'/\'\\\'\'}'"
        else
            line+=" $arg"
        fi
    done
    printf '%s\n' "$line"
    "$@" | tr -d '\r'
    printf '\n\n'
}

echo "########## HTTP :8080 ##########"
echo
run curl -s -X POST "$BASE/medicines" -H 'Content-Type: application/json' \
    -d '{"name":"Парацетамол","manufacturer":"Фармстандарт","price":45.9,"prescription":false,"quantity":100}'

ID=$(curl -s -X POST "$BASE/medicines" -H 'Content-Type: application/json' \
    -d '{"name":"Амоксициллин","manufacturer":"Sandoz","price":245.0,"prescription":true,"quantity":20}' | jq -r .id)
printf '$ ID=$(curl -s -X POST %s/medicines ... | jq -r .id)\n$ echo $ID\n%s\n\n' "$BASE" "$ID"

run curl -s "$BASE/medicines/$ID"
run curl -s -i -X POST "$BASE/medicines/$ID/sell" -H 'Content-Type: application/json' -d '{"count":5}'
run curl -s "$BASE/medicines/$ID"
run curl -s -i -X POST "$BASE/medicines/$ID/sell" -H 'Content-Type: application/json' -d '{"count":50}'
run curl -s -i "$BASE/medicines/no-such-id"
run curl -s -X POST "$BASE/medicines" -H 'Content-Type: application/json' \
    -d '{"name":"","manufacturer":"X","price":-1,"quantity":1}'

echo "########## gRPC :9090 ##########"
echo
run "$GRPCURL" -plaintext "$GRPC" list
run "$GRPCURL" -plaintext "$GRPC" describe pharmacy.v1.Pharmacy
run "$GRPCURL" -plaintext -d '{"medicine":{"id":"ins-100","name":"Инсулин","manufacturer":"Novo Nordisk","price":1290,"prescription":true,"quantity":10}}' \
    "$GRPC" pharmacy.v1.Pharmacy/Create
run "$GRPCURL" -plaintext -d '{"id":"ins-100","count":3}' "$GRPC" pharmacy.v1.Pharmacy/Sell
run "$GRPCURL" -plaintext -d '{"id":"ins-100"}' "$GRPC" pharmacy.v1.Pharmacy/GetByID

# Данные общие для обоих транспортов: лекарство, созданное по HTTP, видно через gRPC.
run "$GRPCURL" -plaintext -d "{\"id\":\"$ID\"}" "$GRPC" pharmacy.v1.Pharmacy/GetByID
run curl -s "$BASE/medicines/ins-100"

# Ошибки отображаются в коды gRPC.
set +e
run "$GRPCURL" -plaintext -d '{"id":"ins-100","count":100}' "$GRPC" pharmacy.v1.Pharmacy/Sell
run "$GRPCURL" -plaintext -d '{"id":"nope"}' "$GRPC" pharmacy.v1.Pharmacy/GetByID
