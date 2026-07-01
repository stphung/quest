#!/usr/bin/env python3
"""Convert ANSI terminal output (tmux capture-pane -e) to a standalone HTML page.

Supports SGR codes emitted by crossterm/ratatui: 16-color, bright, 256-color
(38;5;n / 48;5;n), truecolor (38;2;r;g;b / 48;2;r;g;b), bold, dim, italic,
underline, and reverse video.

Usage: ansi2html.py [input.txt] > out.html   (reads stdin if no file given)
"""

import html
import re
import sys

# Default terminal palette (VGA-ish, matches what most reviewers expect).
BASE16 = [
    "#000000", "#cd3131", "#0dbc79", "#e5e510",
    "#2472c8", "#bc3fbc", "#11a8cd", "#e5e5e5",
    "#666666", "#f14c4c", "#23d18b", "#f5f543",
    "#3b8eea", "#d670d6", "#29b8db", "#ffffff",
]

DEFAULT_FG = "#d4d4d4"
DEFAULT_BG = "#1e1e1e"


def color_256(n: int) -> str:
    if n < 16:
        return BASE16[n]
    if n < 232:  # 6x6x6 cube
        n -= 16
        r, g, b = n // 36, (n % 36) // 6, n % 6
        conv = [0, 95, 135, 175, 215, 255]
        return f"#{conv[r]:02x}{conv[g]:02x}{conv[b]:02x}"
    gray = 8 + (n - 232) * 10  # grayscale ramp
    return f"#{gray:02x}{gray:02x}{gray:02x}"


class Pen:
    def __init__(self):
        self.reset()

    def reset(self):
        self.fg = None
        self.bg = None
        self.bold = False
        self.dim = False
        self.italic = False
        self.underline = False
        self.reverse = False

    def style(self) -> str:
        fg = self.fg or DEFAULT_FG
        bg = self.bg or DEFAULT_BG
        if self.reverse:
            fg, bg = bg, fg
        parts = [f"color:{fg}"]
        if bg != DEFAULT_BG:
            parts.append(f"background-color:{bg}")
        if self.bold:
            parts.append("font-weight:bold")
        if self.dim:
            parts.append("opacity:0.6")
        if self.italic:
            parts.append("font-style:italic")
        if self.underline:
            parts.append("text-decoration:underline")
        return ";".join(parts)

    def apply(self, params):
        i = 0
        while i < len(params):
            p = params[i]
            if p in (0, None):
                self.reset()
            elif p == 1:
                self.bold = True
            elif p == 2:
                self.dim = True
            elif p == 3:
                self.italic = True
            elif p == 4:
                self.underline = True
            elif p == 7:
                self.reverse = True
            elif p == 22:
                self.bold = self.dim = False
            elif p == 23:
                self.italic = False
            elif p == 24:
                self.underline = False
            elif p == 27:
                self.reverse = False
            elif 30 <= p <= 37:
                self.fg = BASE16[p - 30]
            elif p == 39:
                self.fg = None
            elif 40 <= p <= 47:
                self.bg = BASE16[p - 40]
            elif p == 49:
                self.bg = None
            elif 90 <= p <= 97:
                self.fg = BASE16[p - 90 + 8]
            elif 100 <= p <= 107:
                self.bg = BASE16[p - 100 + 8]
            elif p in (38, 48):
                target = "fg" if p == 38 else "bg"
                if i + 1 < len(params) and params[i + 1] == 5 and i + 2 < len(params):
                    setattr(self, target, color_256(params[i + 2]))
                    i += 2
                elif i + 1 < len(params) and params[i + 1] == 2 and i + 4 < len(params):
                    r, g, b = params[i + 2], params[i + 3], params[i + 4]
                    setattr(self, target, f"#{r:02x}{g:02x}{b:02x}")
                    i += 4
            i += 1


ANSI_RE = re.compile(r"\x1b\[([0-9;]*)m|\x1b\[[0-9;?]*[a-lnzA-Z]|\x1b[()][A-Z0-9]")


def convert(text: str) -> str:
    pen = Pen()
    out = []
    for line in text.split("\n"):
        pos = 0
        for m in ANSI_RE.finditer(line):
            chunk = line[pos:m.start()]
            if chunk:
                out.append(f'<span style="{pen.style()}">{html.escape(chunk)}</span>')
            if m.group(1) is not None:
                params = [int(x) if x else 0 for x in m.group(1).split(";")] if m.group(1) else [0]
                pen.apply(params)
            pos = m.end()
        chunk = line[pos:]
        if chunk:
            out.append(f'<span style="{pen.style()}">{html.escape(chunk)}</span>')
        out.append("\n")
    return "".join(out)


def self_test():
    # 16-color foreground
    out = convert("\x1b[31mred\x1b[0m")
    assert 'color:#cd3131">red</span>' in out, out
    # bright background + reverse swaps fg/bg
    out = convert("\x1b[7;93mrev\x1b[0m")
    assert f"color:{DEFAULT_BG}" in out and "background-color:#f5f543" in out, out
    # 256-color: cube entries 196 = #ff0000, 160 = #d70000; grayscale 244 = #808080
    out = convert("\x1b[38;5;196mx\x1b[38;5;160m\x1b[48;5;244my")
    assert "color:#ff0000" in out and "color:#d70000" in out, out
    assert "background-color:#808080" in out, out
    # truecolor
    out = convert("\x1b[38;2;18;52;86mz")
    assert "color:#123456" in out, out
    # bold resets with 22; text and HTML escaping survive
    out = convert("\x1b[1mB\x1b[22m<&>")
    assert "font-weight:bold" in out and "&lt;&amp;&gt;" in out, out
    # non-SGR sequences (cursor moves, charset) are stripped
    out = convert("\x1b[2Jclean\x1b(B\x1b[1;5H")
    assert ">clean</span>" in out and "\x1b" not in out, out
    # multi-line output preserves newlines for <pre>
    assert convert("a\nb").count("\n") == 2
    print("ansi2html self-test passed")


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
        self_test()
        return
    text = open(sys.argv[1], encoding="utf-8", errors="replace").read() if len(sys.argv) > 1 else sys.stdin.read()
    body = convert(text)
    print(f"""<!DOCTYPE html>
<html><head><meta charset="utf-8"><style>
  html, body {{ margin: 0; padding: 0; background: {DEFAULT_BG}; }}
  pre {{
    margin: 0; padding: 12px;
    font-family: "DejaVu Sans Mono", "Menlo", "Consolas", monospace;
    font-size: 14px; line-height: 1.25;
    background: {DEFAULT_BG}; color: {DEFAULT_FG};
    display: inline-block;
  }}
</style></head><body><pre>{body}</pre></body></html>""")


if __name__ == "__main__":
    main()
