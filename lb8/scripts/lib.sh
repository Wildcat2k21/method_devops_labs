# Общие функции для скриптов ЛР 8 (подключается через `source`).

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
    "$@" 2>&1 | tr -d '\r'
    printf '\n'
}

# Ждёт, пока команда не завершится успешно (не более N попыток по 0.5 с).
wait_for() {
    local tries=$1
    shift
    for ((i = 0; i < tries; i++)); do
        "$@" >/dev/null 2>&1 && return 0
        sleep 0.5
    done
    echo "не дождались: $*" >&2
    return 1
}
