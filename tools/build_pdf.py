#!/usr/bin/env python3
"""Конвертирует отчёты lb*/README.md в PDF через PrinceXML.

Markdown -> HTML (markdown-it-py + подсветка Pygments) -> PDF (prince).

    python3 tools/build_pdf.py            # все отчёты
    python3 tools/build_pdf.py lb1 lb6    # только указанные работы
"""

from __future__ import annotations

import html
import re
import subprocess
import sys
import tempfile
from pathlib import Path

from markdown_it import MarkdownIt
from pygments import highlight
from pygments.formatters import HtmlFormatter
from pygments.lexers import TextLexer, get_lexer_by_name
from pygments.util import ClassNotFound

ROOT = Path(__file__).resolve().parent.parent

CSS = """
@page {
  size: A4;
  margin: 18mm 16mm 20mm 18mm;
  @bottom-right { content: counter(page) " / " counter(pages); font-size: 9pt; color: #666; }
  @bottom-left { content: string(title); font-size: 9pt; color: #666; }
}
html { font-family: "Noto Serif", "DejaVu Serif", serif; font-size: 11pt; line-height: 1.45; color: #1a1a1a; }
h1 { string-set: title content(); font-size: 18pt; margin: 14pt 0 10pt; border-bottom: 2px solid #2b6cb0; padding-bottom: 4pt; }
h2 { font-size: 14.5pt; margin: 16pt 0 6pt; color: #1e4e8c; page-break-after: avoid; }
h3 { font-size: 12.5pt; margin: 12pt 0 4pt; page-break-after: avoid; }
h4 { font-size: 11pt; margin: 10pt 0 4pt; page-break-after: avoid; }
h1, h2, h3, h4 { font-family: "Noto Sans", "DejaVu Sans", sans-serif; }
p { margin: 4pt 0 6pt; text-align: justify; hyphens: auto; }
a { color: #2b6cb0; text-decoration: none; }
code { font-family: "Noto Sans Mono", "DejaVu Sans Mono", monospace; font-size: 9pt; background: #f2f4f7; padding: 0 2pt; border-radius: 2pt; }
pre { font-family: "Noto Sans Mono", "DejaVu Sans Mono", monospace; font-size: 8.3pt; line-height: 1.3;
      background: #f6f8fa; border: 1px solid #d8dee4; border-radius: 3pt; padding: 6pt 8pt;
      white-space: pre-wrap; word-wrap: break-word; }
pre code { background: none; padding: 0; font-size: inherit; }
table { border-collapse: collapse; margin: 6pt 0 10pt; font-size: 9.5pt; width: 100%; }
th, td { border: 1px solid #c8d0d8; padding: 3pt 6pt; vertical-align: top; }
th { background: #eef2f7; }
img { max-width: 100%; display: block; margin: 8pt auto; }
blockquote { margin: 6pt 0; padding: 2pt 10pt; border-left: 3px solid #2b6cb0; color: #444; background: #f7f9fc; }
hr { border: none; border-top: 1px solid #ccd; margin: 12pt 0; }
"""


def _highlight(code: str, lang: str, _attrs: str) -> str:
    try:
        lexer = get_lexer_by_name(lang) if lang else TextLexer()
    except ClassNotFound:
        lexer = TextLexer()
    return highlight(code, lexer, HtmlFormatter(nowrap=True))


def render(md_path: Path) -> Path:
    md = MarkdownIt("commonmark", {"html": True, "highlight": _highlight}).enable(
        ["table", "strikethrough"]
    )
    body = md.render(md_path.read_text(encoding="utf-8"))
    # Бейджи GitHub Actions в PDF бесполезны (и требуют сети) — убираем.
    body = re.sub(r'<a [^>]*><img [^>]*badge\.svg[^>]*></a>', "", body)
    title = html.escape(md_path.parent.name)
    pygments_css = HtmlFormatter(style="friendly").get_style_defs("pre")
    doc = (
        f'<!doctype html><html lang="ru"><head><meta charset="utf-8">'
        f"<title>{title}</title><style>{CSS}\n{pygments_css}</style></head>"
        f"<body>{body}</body></html>"
    )
    out = md_path.with_suffix(".pdf")
    with tempfile.NamedTemporaryFile(
        "w", suffix=".html", dir=md_path.parent, delete=True, encoding="utf-8"
    ) as tmp:
        tmp.write(doc)
        tmp.flush()
        subprocess.run(
            ["prince", "--no-network", tmp.name, "-o", str(out)],
            check=True,
        )
    return out


def main(labs: list[str]) -> int:
    readmes = [ROOT / lab / "README.md" for lab in labs] or sorted(ROOT.glob("lb*/README.md"))
    for md_path in readmes:
        out = render(md_path)
        print(f"{md_path.relative_to(ROOT)} -> {out.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
