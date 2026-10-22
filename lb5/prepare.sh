#!/usr/bin/env bash
# ЛР 5: подготовка рабочих каталогов и файлов (шаги 2–4).
# Git-часть работы (шаги 7–11) выполнена в репозитории воркспейса.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
LAB=${LAB:-$HOME/devops/lab5}

# Шаг 2: структура каталогов
mkdir -p "$LAB/git" "$LAB/linux"

# Шаг 3: commands.txt — только echo, cat и перенаправление вывода
cd "$LAB/linux"
echo "Изученные команды Linux" > commands.txt
echo "=======================" >> commands.txt
echo "pwd    - вывести абсолютный путь текущего каталога" >> commands.txt
echo "ls     - список файлов каталога (-l подробно, -a со скрытыми)" >> commands.txt
echo "cd     - перейти в другой каталог (cd ~ - домой, cd .. - на уровень выше)" >> commands.txt
echo "mkdir  - создать каталог (-p вместе с родительскими)" >> commands.txt
cat >> commands.txt <<'EOF'
touch  - создать пустой файл или обновить время изменения
echo   - вывести строку; > перезаписать файл, >> дописать в конец
cat    - вывести содержимое файла (или склеить несколько файлов)
chmod  - изменить права доступа (chmod +x file - сделать исполняемым)
grep   - найти строки по шаблону (-r рекурсивно, -n с номерами строк)
find   - найти файлы по имени, типу, размеру (find ~/devops -name "*.txt")
whoami - имя текущего пользователя Linux
date   - текущие дата и время
hostname - имя хоста
df     - свободное место на дисках (-h в удобных единицах)
git    - распределённая система контроля версий
Linux - семейство ОС на ядре Linux; все команды выше входят в стандартный набор Linux
EOF

# Шаг 4: info.sh
cp "$HERE/files/linux/info.sh" "$LAB/linux/info.sh"
chmod +x "$LAB/linux/info.sh"

echo "Готово: $LAB"
