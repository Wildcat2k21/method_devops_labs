#!/usr/bin/env bash
# ЛР 5, шаг 4: сведения о пользователе, времени, хосте и свободном месте.

echo "Пользователь:     $(whoami)"
echo "Дата и время:     $(date '+%Y-%m-%d %H:%M:%S %Z')"
echo "Имя хоста:        $(hostname)"
echo "Свободно на /:    $(df -h --output=avail / | tail -n 1 | tr -d ' ')"
echo "Свободно в \$HOME: $(df -h --output=avail "$HOME" | tail -n 1 | tr -d ' ')"
