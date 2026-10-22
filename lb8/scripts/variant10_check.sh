#!/usr/bin/env bash
# ЛР 8, вариант 10: убеждаемся, что файл на томе crondata растёт каждую минуту.
# Предварительно (см. lb8/cron/Dockerfile):
#   docker build -t lb8-cron lb8/cron
#   docker volume create crondata
#   docker run -d --name cron-writer -v crondata:/data lb8-cron
set -euo pipefail
source "$(dirname "$0")/lib.sh"

INTERVAL=${INTERVAL:-120}

run docker ps --filter name=cron-writer
run docker exec cron-writer cat /etc/crontabs/root
run docker exec cron-writer wc -l /data/dates.log
run docker exec cron-writer tail -n 3 /data/dates.log

echo "... ждём ${INTERVAL} секунд ..."
echo
sleep "$INTERVAL"

run docker exec cron-writer wc -l /data/dates.log
run docker exec cron-writer tail -n 3 /data/dates.log
# Файл лежит на томе, а не в контейнере: его видит любой другой контейнер.
run docker run --rm -v crondata:/data alpine:3.24 sh -c 'ls -l /data && head -n 3 /data/dates.log'
run docker logs --tail 4 cron-writer
run docker volume inspect crondata
