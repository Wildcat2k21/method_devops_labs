#!/usr/bin/env bash
# ЛР 8, задание 1: базовые команды Docker — образы, контейнеры, порты, тома, сети.
set -euo pipefail
source "$(dirname "$0")/lib.sh"

run docker version
run docker run hello-world

run docker pull nginx
run docker images

run docker run -d --name my-nginx -p 8080:80 nginx
wait_for 20 curl -fs http://localhost:8080
run docker ps
run curl -s http://localhost:8080
run docker logs my-nginx

run docker network create mynet
run docker run -d --name web --network mynet nginx
run docker exec web hostname

run docker volume create pgdata
run docker volume inspect pgdata
run docker network inspect mynet
