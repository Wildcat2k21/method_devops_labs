#!/usr/bin/env bash
# ЛР 8, задание 3: данные PostgreSQL переживают удаление контейнера,
# потому что хранятся в именованном томе pgdata.
set -euo pipefail
source "$(dirname "$0")/lib.sh"

PG_ENV=(-e POSTGRES_PASSWORD=lab8)
PG_VOL=(-v pgdata:/var/lib/postgresql/data)

run docker volume create pgdata
run docker run -d --name pg1 "${PG_ENV[@]}" "${PG_VOL[@]}" postgres:16
# Ждём «настоящий» сервер: во время инициализации временный сервер слушает
# только unix-сокет, поэтому проверяем по TCP.
wait_for 120 docker exec pg1 pg_isready -h 127.0.0.1 -U postgres

run docker exec pg1 psql -U postgres -c \
    "CREATE TABLE medicines (id serial PRIMARY KEY, name text NOT NULL, quantity int NOT NULL);"
run docker exec pg1 psql -U postgres -c \
    "INSERT INTO medicines (name, quantity) VALUES ('Парацетамол', 100), ('Ибупрофен', 40);"
run docker exec pg1 psql -U postgres -c "SELECT * FROM medicines;"

run docker rm -f pg1
run docker ps -a --filter name=pg1
run docker volume ls --filter name=pgdata

run docker run -d --name pg2 "${PG_ENV[@]}" "${PG_VOL[@]}" postgres:16
wait_for 120 docker exec pg2 pg_isready -h 127.0.0.1 -U postgres
run docker exec pg2 psql -U postgres -c '\dt'
run docker exec pg2 psql -U postgres -c "SELECT * FROM medicines;"
run docker volume inspect pgdata

run docker rm -f pg2
