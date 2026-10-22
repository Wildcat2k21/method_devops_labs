#!/usr/bin/env bash
# ЛР 6, задание 1: проверка трёх эндпоинтов order-service через curl.
# Перед запуском: cargo run -p lb6 --bin orders
set -euo pipefail

BASE=${BASE:-http://localhost:8080}

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

run curl -s -X POST "$BASE/orders" -H 'Content-Type: application/json' \
    -d '{"customer":"Альберт","item":"Ноутбук","quantity":1}'
run curl -s -X POST "$BASE/orders" -H 'Content-Type: application/json' \
    -d '{"customer":"Мария","item":"Монитор","quantity":2}'
run curl -s -i "$BASE/orders/1"
run curl -s "$BASE/orders"
run curl -s -i "$BASE/orders/42"
run curl -s -X POST "$BASE/orders" -H 'Content-Type: application/json' -d '{"customer":"Олег"}'
