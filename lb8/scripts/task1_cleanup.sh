#!/usr/bin/env bash
# ЛР 8, задание 1: занятое место до и после очистки.
#
# Удаляются только объекты, созданные в задании 1. `docker system prune`
# намеренно не используется: на рабочей машине он удалил бы и чужие
# остановленные контейнеры, неиспользуемые образы и (с --volumes) данные.
set -euo pipefail
source "$(dirname "$0")/lib.sh"

run docker system df

run docker rm -f my-nginx web
# Остановленные контейнеры, оставшиеся после `docker run hello-world`.
mapfile -t hello < <(docker ps -aq --filter ancestor=hello-world)
if ((${#hello[@]})); then run docker rm "${hello[@]}"; fi
run docker network rm mynet
run docker volume rm pgdata
run docker rmi nginx
run docker image prune -f

run docker system df
