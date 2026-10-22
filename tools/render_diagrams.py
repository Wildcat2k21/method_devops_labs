#!/usr/bin/env python3
"""Рендерит все диаграммы PlantUML (lb*/docs/*.puml) в SVG.

Использует локальный PlantUML Server (docker-образ plantuml/plantuml-server),
адрес задаётся переменной PLANTUML_URL (по умолчанию http://127.0.0.1:8080).

    python3 tools/render_diagrams.py            # все диаграммы
    python3 tools/render_diagrams.py lb6 lb8    # только указанные работы
"""

from __future__ import annotations

import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SERVER = os.environ.get("PLANTUML_URL", "http://127.0.0.1:8080").rstrip("/")


def render(src: Path) -> Path:
    req = urllib.request.Request(
        f"{SERVER}/svg",
        data=src.read_bytes(),
        headers={"Content-Type": "text/plain; charset=utf-8"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=60) as resp:
        svg = resp.read()
    # Сервер возвращает SVG и при синтаксической ошибке — с текстом ошибки внутри.
    if b"Syntax Error" in svg:
        raise RuntimeError(f"синтаксическая ошибка PlantUML в {src.relative_to(ROOT)}")
    out = src.with_suffix(".svg")
    out.write_bytes(svg)
    return out


def main(labs: list[str]) -> int:
    patterns = [f"{lab}/docs/*.puml" for lab in labs] or ["lb*/docs/*.puml"]
    sources = sorted({p for pat in patterns for p in ROOT.glob(pat)})
    if not sources:
        print("диаграммы не найдены", file=sys.stderr)
        return 1
    failed = 0
    for src in sources:
        try:
            out = render(src)
            print(f"{src.relative_to(ROOT)} -> {out.name}")
        except (urllib.error.URLError, RuntimeError) as e:
            failed += 1
            print(f"ОШИБКА {src.relative_to(ROOT)}: {e}", file=sys.stderr)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
