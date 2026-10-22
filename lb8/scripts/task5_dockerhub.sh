#!/usr/bin/env bash
# ЛР 8, задание 5: публикация order-service в Docker Hub и запуск из реестра.
# Требуется выполненный `docker login`. Логин передаётся в DOCKERHUB_USER.
set -euo pipefail
source "$(dirname "$0")/lib.sh"

USER_NAME=${DOCKERHUB_USER:?укажите DOCKERHUB_USER}
IMAGE="$USER_NAME/order-service:v1"

run docker tag order-service:v1 "$IMAGE"
run docker images order-service
run docker images "$USER_NAME/order-service"
run docker push "$IMAGE"

# Имитация «другой машины»: удаляем все локальные копии образа.
run docker rmi "$IMAGE" order-service:v1
run docker images --filter reference='*order-service*'

run docker pull "$IMAGE"
run docker run -d --rm --name order-service -p 8082:8080 "$IMAGE"
wait_for 40 curl -fs http://localhost:8082/orders
run docker ps --filter name=order-service
run curl -s -X POST http://localhost:8082/orders -H 'Content-Type: application/json' \
    -d '{"customer":"Альберт","item":"Ноутбук","quantity":1}'
run curl -s http://localhost:8082/orders/1
run curl -s http://localhost:8082/orders
run docker logs order-service
run docker stop order-service
