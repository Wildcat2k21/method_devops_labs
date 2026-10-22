#!/usr/bin/env bash
# ЛР 8, задание 4: связка двух контейнеров в пользовательской сети appnet.
# Во встроенном DNS Docker (127.0.0.11) имя контейнера резолвится в его IP.
set -euo pipefail
source "$(dirname "$0")/lib.sh"

run docker network create appnet
run docker run -d --name db --network appnet -e POSTGRES_PASSWORD=lab8 postgres:16
run docker run -d --name api --network appnet pharmacy-service:v1
wait_for 120 docker exec db pg_isready -h 127.0.0.1 -U postgres

run docker ps --filter network=appnet
run docker exec api ping -c 3 db
run docker exec api nslookup db
run docker exec api cat /etc/resolv.conf
# Обратная проверка: api доступен из db по имени (HTTP через bash /dev/tcp).
run docker exec db bash -c \
    'exec 3<>/dev/tcp/api/8080; printf "GET /health HTTP/1.1\r\nHost: api\r\nConnection: close\r\n\r\n" >&3; cat <&3'
run docker network inspect appnet --format '{{range .Containers}}{{.Name}} {{.IPv4Address}}{{"\n"}}{{end}}'

run docker rm -f api db
run docker network rm appnet
