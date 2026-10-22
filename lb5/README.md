**Имя студента:** Цароев Альберт Казебкович\
**Группа:** ПИЖ-б-о-23-1(1)\
**Предмет:** Контейнерные технологии и DevOps для ИТ-специалистов\
**Претендую на оценку:** Отлично

# Лабораторная работа №5. Основы Linux и Git

## Цель работы

Освоить базовые команды Linux для работы с файловой системой, файлами,
правами доступа и поиском, научиться писать простые shell-скрипты, а также
изучить основной цикл работы с Git: настройку, коммиты, ветки, слияние,
игнорирование файлов, разрешение конфликтов и публикацию на GitHub.

**Ссылка на удалённый репозиторий:** <ССЫЛКА НА РЕПОЗИТОРИЙ>

## Задание

1. Установить Linux в виртуальной машине или использовать WSL 2.
2. Выполнить команды навигации `pwd`, `ls`, `cd`; создать каталоги
   `~/devops/lab5/git` и `~/devops/lab5/linux`.
3. Создать в каталоге `linux` файл `commands.txt` со списком изученных команд,
   используя `echo`, `cat` и перенаправление вывода.
4. Создать скрипт `info.sh`, выводящий имя пользователя, дату и время, имя
   хоста и свободное место на диске; сделать его исполняемым и запустить.
5. С помощью `grep` и `find` найти все `.txt` в `~/devops` и вывести строки со
   словом Linux.
6. Настроить Git: имя пользователя и email.
7. Создать репозиторий в `~/devops/lab5/git`, добавить `README.md`, выполнить
   первый коммит.
8. Создать ветку `feature/commands`, добавить `linux-commands.md`, закоммитить,
   слить с `main`.
9. Создать на GitHub репозиторий `devops-lab5` и отправить в него локальный
   репозиторий.
10. Создать `.gitignore`, исключающий `*.log`, `*.tmp` и каталог `.venv`;
    проверить, что такие файлы не попадают в индекс.
11. Смоделировать и вручную разрешить конфликт слияния.
12. Оформить отчёт.

## Ход работы

Файлы работы подготовлены в каталоге `lb5/` воркспейса:

```text
lb5/
├── README.md                   — этот отчёт
├── prepare.sh                  — создание структуры ~/devops/lab5 и файлов (шаги 2–4, 7)
├── files/
│   ├── linux/info.sh           — скрипт шага 4
│   └── git/
│       ├── README.md           — описание работы для репозитория devops-lab5
│       ├── linux-commands.md   — файл для ветки feature/commands (шаг 8)
│       └── .gitignore          — шаг 10
└── screenshots/                — скриншоты ключевых этапов
```

### Шаг 1. Операционная система

Работа выполняется в Ubuntu 26.04 LTS, установленной на компьютер (нативно,
без виртуальной машины).

```bash
$ cat /etc/os-release | head -4
PRETTY_NAME="Ubuntu 26.04 LTS"
NAME="Ubuntu"
VERSION_ID="26.04"
VERSION="26.04 LTS (Resolute Raccoon)"
$ uname -srm
Linux 7.0.0-38-generic x86_64
```

![ОС](screenshots/01_os.png)

### Шаг 2. Навигация и структура каталогов

| Команда                                       | Пояснение                                                   |
|-----------------------------------------------|-------------------------------------------------------------|
| `pwd`                                         | вывести абсолютный путь текущего каталога                   |
| `ls`, `ls -l`, `ls -la`                       | список файлов; `-l` — подробно, `-a` — включая скрытые      |
| `cd ~`, `cd ..`, `cd ~/devops/lab5`           | перейти в домашний, родительский или указанный каталог      |
| `mkdir -p ~/devops/lab5/git ~/devops/lab5/linux` | создать каталоги вместе с отсутствующими родительскими   |

![Навигация](screenshots/02_navigation.png)

### Шаг 3. Файл commands.txt

Файл собирается только средствами оболочки: первая строка записывается с
перезаписью (`>`), следующие дописываются (`>>`), а блок из нескольких строк
добавляется командой `cat` с heredoc:

```bash
cd ~/devops/lab5/linux
echo "Изученные команды Linux" > commands.txt
echo "=======================" >> commands.txt
echo "pwd    - вывести абсолютный путь текущего каталога" >> commands.txt
# ...
cat >> commands.txt <<'EOF'
touch  - создать пустой файл или обновить время изменения
echo   - вывести строку; > перезаписать файл, >> дописать в конец
# ...
EOF
cat commands.txt
```

```text
Изученные команды Linux
=======================
pwd    - вывести абсолютный путь текущего каталога
ls     - список файлов каталога (-l подробно, -a со скрытыми)
cd     - перейти в другой каталог (cd ~ - домой, cd .. - на уровень выше)
mkdir  - создать каталог (-p вместе с родительскими)
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
```

![commands.txt](screenshots/03_commands_txt.png)

### Шаг 4. Скрипт info.sh

```bash
#!/usr/bin/env bash
# ЛР 5, шаг 4: сведения о пользователе, времени, хосте и свободном месте.

echo "Пользователь:     $(whoami)"
echo "Дата и время:     $(date '+%Y-%m-%d %H:%M:%S %Z')"
echo "Имя хоста:        $(hostname)"
echo "Свободно на /:    $(df -h --output=avail / | tail -n 1 | tr -d ' ')"
echo "Свободно в \$HOME: $(df -h --output=avail "$HOME" | tail -n 1 | tr -d ' ')"
```

Строка `#!/usr/bin/env bash` (shebang) указывает интерпретатор. `$(…)` —
подстановка вывода команды. `df --output=avail` выводит только столбец
свободного места, а `tail -n 1` отбрасывает заголовок. Чтобы скрипт можно
было запускать как программу (`./info.sh`), ему нужно право на исполнение:
`chmod +x info.sh` (права меняются с `-rw-rw-r--` на `-rwxrwxr-x`).

```bash
$ chmod +x info.sh
$ ./info.sh
Пользователь:     origami
Дата и время:     2026-10-22 09:16:30 MSK
Имя хоста:        origami-rlefxx
Свободно на /:    23G
Свободно в $HOME: 23G
```

![info.sh](screenshots/04_info_sh.png)

### Шаг 5. Поиск файлов и строк

`find` находит файлы по критериям, `-exec … {} +` передаёт найденные файлы
команде `grep`. Ключ `-H` выводит имя файла, `-n` — номер строки:

```bash
$ find ~/devops -type f -name "*.txt" -exec grep -Hn "Linux" {} +
/home/origami/devops/lab5/linux/commands.txt:1:Изученные команды Linux
/home/origami/devops/lab5/linux/commands.txt:13:whoami - имя текущего пользователя Linux
/home/origami/devops/lab5/linux/commands.txt:18:Linux - семейство ОС на ядре Linux; все команды выше входят в стандартный набор Linux
```

Тот же результат даёт один `grep`: `grep -rn --include='*.txt' Linux ~/devops`.

![find и grep](screenshots/05_find_grep.png)

### Шаг 6. Настройка Git

```bash
git config --global user.name  "chainwarden771"
git config --global user.email "lickmysh0es@yandex.ru"
git config --global --list | grep user
```

Ключ `--global` сохраняет настройки в `~/.gitconfig` для всех репозиториев
пользователя. Имя и email записываются в каждый коммит как автор.

![git config](screenshots/06_git_config.png)

### Шаг 7. Репозиторий и первый коммит

```bash
cd ~/devops/lab5/git
git init -b main          # создать репозиторий с основной веткой main
git status                # README.md — неотслеживаемый файл
git add README.md         # добавить в индекс (staging area)
git commit -m "docs: описание лабораторной работы №5"
git log --oneline
```

`git add` переносит изменения в индекс, `git commit` фиксирует содержимое
индекса как новый снимок (коммит) с автором, датой и сообщением.

![Первый коммит](screenshots/07_first_commit.png)

### Шаг 8. Ветка feature/commands и слияние

```bash
git switch -c feature/commands      # создать ветку и перейти в неё
cp …/lb5/files/git/linux-commands.md .
git add linux-commands.md
git commit -m "docs: список команд Linux и Git"
git switch main
git merge feature/commands          # fast-forward: main не менялась после ответвления
git log --oneline --graph --all
```

Так как после создания ветки в `main` не было новых коммитов, слияние
выполняется перемоткой (fast-forward): указатель `main` просто сдвигается на
коммит ветки, отдельный коммит слияния не нужен.

![Ветка и слияние](screenshots/08_feature_branch.png)

### Шаг 9. Публикация на GitHub

На GitHub создан пустой публичный репозиторий `devops-lab5` (без README, чтобы
истории не расходились). Затем локальный репозиторий связан с удалённым и
отправлен:

```bash
git remote add origin https://github.com/<логин>/devops-lab5.git
git push -u origin main             # -u запоминает origin/main как upstream
git push origin feature/commands
git remote -v
```

![git push](screenshots/09_github_push.png)

![Репозиторий на GitHub](screenshots/09_github_page.png)

### Шаг 10. Файл .gitignore

```gitignore
# Журналы и временные файлы
*.log
*.tmp

# Виртуальное окружение Python
.venv/
```

Проверка: создаются файлы, подпадающие под правила, и выполняется `git add .`.
В индекс попадает только `.gitignore`, а `git check-ignore -v` показывает,
каким правилом исключён каждый файл.

```bash
touch debug.log cache.tmp
mkdir -p .venv && touch .venv/pyvenv.cfg
git status --short
git add .
git status --short
git check-ignore -v debug.log cache.tmp .venv/pyvenv.cfg
git commit -m "chore: .gitignore для *.log, *.tmp и .venv"
```

![.gitignore](screenshots/10_gitignore.png)

### Шаг 11. Конфликт слияния

Две ветки изменяют одну и ту же (первую) строку `README.md` по-разному:

```bash
git switch -c conflict-a
sed -i '1s/.*/# devops-lab5: Linux и Git/' README.md
git commit -am "docs: заголовок README (ветка conflict-a)"

git switch main
git switch -c conflict-b
sed -i '1s/.*/# devops-lab5 — лабораторная работа №5/' README.md
git commit -am "docs: заголовок README (ветка conflict-b)"

git switch main
git merge conflict-a      # fast-forward
git merge conflict-b      # CONFLICT (content): Merge conflict in README.md
```

Git не может выбрать между двумя версиями строки и помечает её маркерами:

```text
<<<<<<< HEAD
# devops-lab5: Linux и Git
=======
# devops-lab5 — лабораторная работа №5
>>>>>>> conflict-b
```

![Конфликт](screenshots/11_conflict.png)

Конфликт разрешён вручную: в редакторе оставлена итоговая строка
`# devops-lab5 — Linux и Git, лабораторная работа №5`, маркеры удалены. Затем
файл помечен как разрешённый и создан коммит слияния:

```bash
git add README.md
git commit -m "merge: разрешён конфликт заголовка README"
git log --oneline --graph --all
git push
```

![Конфликт разрешён](screenshots/11_conflict_resolved.png)

## Результаты

* Создана структура `~/devops/lab5/{git,linux}`, файл `commands.txt` и
  исполняемый скрипт `info.sh`; поиск `find` + `grep` нашёл 3 строки со словом
  Linux.
* Git настроен; в репозитории `devops-lab5` есть первый коммит, ветка
  `feature/commands`, слитая в `main`, `.gitignore` и коммит слияния с
  разрешённым конфликтом.
* Репозиторий опубликован на GitHub: <ССЫЛКА НА РЕПОЗИТОРИЙ>.

## Вывод

В ходе работы освоены базовые инструменты командной строки Linux: навигация
(`pwd`, `ls`, `cd`, `mkdir`), работа с файлами через перенаправление вывода
(`>`, `>>`, heredoc), права доступа (`chmod`), поиск (`find`, `grep`) и
написание shell-скриптов с подстановкой команд.

На практике пройден полный цикл работы с Git: настройка автора, индекс и
коммиты, ветки и два вида слияния — перемотка (fast-forward), когда ветки не
расходились, и слияние с конфликтом, когда обе ветки изменили одну строку.
Конфликт Git не разрешает сам: он помечает спорный участок маркерами, а итоговое
содержимое выбирает человек. Файл `.gitignore` защищает репозиторий от
журналов, временных файлов и виртуальных окружений, а `git check-ignore -v`
позволяет проверить, какое правило сработало. Публикация на GitHub через
`git push -u` связывает локальную ветку с удалённой, и это основа для CI/CD в
следующих работах.
