# ЛР 8, вариант 10: crond в Alpine каждую минуту дописывает дату в файл
# на именованном томе.
#
#   docker build -t lb8-cron lb8/cron
#   docker run -d --name cron-writer -v crondata:/data lb8-cron
FROM alpine:3.24

# busybox crond уже есть в базовом образе; tzdata нужен для местного времени.
RUN apk add --no-cache tzdata
ENV TZ=Europe/Moscow

COPY crontab /etc/crontabs/root

VOLUME /data

# -f — на переднем плане (PID 1 контейнера), -d 5 — лог в stderr (виден в
# `docker logs`) с уровнем, при котором записывается каждый запуск задания.
CMD ["crond", "-f", "-d", "5"]
