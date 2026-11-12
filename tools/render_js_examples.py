#!/usr/bin/env python3
"""Render the README GIFs for weave.js from real samples of the public JS API.

From the repo root (needs node and Pillow):
    WEAVE_DOCS_FONT=/path/to/DejaVuSans.ttf python tools/render_js_examples.py
or: nix develop .#docs --command python tools/render_js_examples.py

The samples come from `node js/tools/samples.js`, which drives Weave.spring /
Weave.enter / Weave.animate with a manual 30 Hz clock. These are documentation
illustrations, not browser recordings.
"""
import json
import os
from pathlib import Path
import subprocess

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
FONT = os.environ["WEAVE_DOCS_FONT"]
BG, CARD, LINE, WHITE, MUTED = "#0c1018", "#141b28", "#283449", "#f1f4fa", "#8e9caf"
COLORS = ["#a5b4fc", "#79dfc1", "#f6c78d", "#e4a6e9"]
WARN = "#f6c78d"
SCALE = 2
W, H = 800, 400


def font(size):
    return ImageFont.truetype(FONT, int(size * SCALE))


def mix(c1, c2, t):
    t = max(0.0, min(1.0, t))
    a = [int(c1[i:i + 2], 16) for i in (1, 3, 5)]
    b = [int(c2[i:i + 2], 16) for i in (1, 3, 5)]
    return "#" + "".join(f"{round(x + (y - x) * t):02x}" for x, y in zip(a, b))


class Frame:
    def __init__(self, tag, title, subtitle):
        self.image = Image.new("RGB", (W * SCALE, H * SCALE), BG)
        self.draw = ImageDraw.Draw(self.image)
        self.text(32, 23, "weave", 18)
        self.text(800 - 32 - self.width(tag, 11), 28, tag, 11, MUTED)
        self.line((32, 61, 768, 61))
        self.text(32, 78, title, 25)
        self.text(32, 118, subtitle, 13, MUTED, mono=True)

    def width(self, value, size):
        return self.draw.textlength(value, font=font(size)) / SCALE

    def text(self, x, y, value, size=14, fill=WHITE, mono=False):
        self.draw.text((x * SCALE, y * SCALE), value, font=font(size), fill=fill)

    def line(self, xy, fill=LINE, width=1):
        self.draw.line(tuple(v * SCALE for v in xy), fill=fill, width=int(width * SCALE))

    def box(self, xy, fill, radius=0, outline=None):
        self.draw.rounded_rectangle(tuple(v * SCALE for v in xy), radius=radius * SCALE,
                                    fill=fill, outline=outline, width=SCALE)

    def done(self):
        return self.image.resize((W, H), Image.Resampling.LANCZOS).quantize(colors=96)


def save(name, frames):
    output = ROOT / "docs/assets" / name
    frames[0].save(output, save_all=True, append_images=frames[1:],
                   duration=[30 if i % 3 else 40 for i in range(len(frames))], loop=0, optimize=True, disposal=2)
    print(f"{output.relative_to(ROOT)}: {output.stat().st_size:,} bytes")


def toggle(s):
    events = {10: "knob.to(1)", 52: "knob.to(0)", 58: "knob.to(1)  ← changed mind", 105: "knob.to(0)"}
    frames, last = [], ""
    for i, v in enumerate(s["toggle"]):
        f = Frame("04 / WEAVE.JS · SPRING", "Flip it. Flip it back.",
                  "const knob = Weave.spring(0, v => ..., { preset: 'expressive' })")
        last = events.get(i, last)
        x0, y0, tw, th = 300, 190, 200, 96
        f.box((x0, y0, x0 + tw, y0 + th), mix(LINE, COLORS[1], v), th / 2)
        r = th / 2 - 8
        cx = x0 + th / 2 + v * (tw - th)
        f.box((cx - r, y0 + th / 2 - r, cx + r, y0 + th / 2 + r), WHITE, r)
        f.text(x0 + tw + 28, y0 + 34, "ON" if v > .5 else "OFF", 18, COLORS[1] if v > .5 else MUTED)
        f.text(32, 330, last, 14, COLORS[1], mono=True)
        f.text(32, 365, "MOMENTUM SURVIVES THE REVERSAL", 11, COLORS[1])
        f.text(575, 365, "Real weave.js output", 11, MUTED)
        frames.append(f.done())
    save("js-toggle.gif", frames)


def stagger(s):
    labels = ["Container 1 · 23.4 °C", "Container 2 · 24.1 °C", "Container 3 · 31.8 °C", "Container 4 · 22.9 °C", "Container 5 · 26.0 °C"]
    frames = []
    for i, cards in enumerate(s["cards"]):
        f = Frame("05 / WEAVE.JS · ENTER", "Lists that arrive in order.",
                  "Weave.enter('.card', { stagger: 0.12 })")
        for n, (dy, op) in enumerate(cards):
            y = 158 + n * 38 + dy
            fill = mix(BG, CARD, op)
            f.box((32, y, 768, y + 30), fill, 8, outline=mix(BG, LINE, op))
            f.box((46, y + 11, 54, y + 19), mix(BG, COLORS[n % 4], op), 4)
            f.text(68, y + 7, labels[n], 13, mix(BG, WHITE, op))
        f.text(32, 365, "ONE SPRING PER PROPERTY", 11, COLORS[0])
        f.text(512, 365, "y + opacity, 0.12 s apart", 11, MUTED)
        frames.append(f.done())
    save("js-stagger.gif", frames)


def counter(s):
    frames = []
    for i, v in enumerate(s["reading"]):
        f = Frame("06 / WEAVE.JS · VALUES", "Numbers that roll, not jump.",
                  "const t = Weave.spring(21.4, v => el.textContent = v.toFixed(1)); t.to(35.2)")
        hot = v >= 30
        f.text(32, 168, f"{v:4.1f}", 84, WARN if hot else WHITE)
        f.text(300, 205, "°C", 26, MUTED)
        f.text(300, 246, "WARM" if hot else "NORMAL", 13, WARN if hot else COLORS[1])
        x0, x1, y = 420, 760, 230
        f.box((x0, y, x1, y + 16), CARD, 8)
        frac = (v - 15) / 25
        f.box((x0, y, x0 + (x1 - x0) * max(.03, min(1, frac)), y + 16), WARN if hot else COLORS[1], 8)
        wx = x0 + (x1 - x0) * (30 - 15) / 25
        f.line((wx, y - 10, wx, y + 26), WARN, 1)
        f.text(wx - 10, y + 32, "30°", 11, MUTED)
        f.text(32, 365, "THE TARGET CHANGES, THE VALUE FOLLOWS", 11, COLORS[2])
        f.text(560, 365, "Default spatial preset", 11, MUTED)
        frames.append(f.done())
    save("js-counter.gif", frames)


def parity(s):
    names = list(s["parity"])
    n = len(s["parity"][names[0]]["rust"])
    diff = max(abs(a - b) for k in names for a, b in zip(s["parity"][k]["rust"], s["parity"][k]["js"]))
    frames = []
    left, right, top, bottom = 38, 760, 165, 330
    for i in range(0, n + 30):
        f = Frame("07 / RUST = JAVASCRIPT", "One engine, two runtimes.",
                  "weave.js (no wasm) vs weaver-motion (Rust), 1/60 s steps")
        for frac in (0, 1):
            y = bottom - frac * (bottom - top) * .8
            f.line((left, y, right, y))
        upto = min(i, n - 1)
        for k, name in enumerate(names):
            rust, js = s["parity"][name]["rust"], s["parity"][name]["js"]
            px = lambda j: left + j / (n - 1) * (right - left)
            py = lambda val: bottom - val * (bottom - top) * .8
            for j in range(1, upto + 1):
                f.line((px(j - 1), py(rust[j - 1]), px(j), py(rust[j])), mix(BG, COLORS[k], .45), 5)
            for j in range(0, upto + 1, 3):
                f.box((px(j) - 2.5, py(js[j]) - 2.5, px(j) + 2.5, py(js[j]) + 2.5), COLORS[k], 2.5)
            f.text(right - 150, 168 + k * 18, name, 11, COLORS[k])
        f.text(32, 365, f"MAX DIFFERENCE: {diff:.1e} (ROUNDING)", 11, COLORS[3])
        f.text(468, 365, "Wide line = Rust · dots = JavaScript", 11, MUTED)
        frames.append(f.done())
    save("js-vs-rust.gif", frames)


def main():
    samples = json.loads(subprocess.check_output(["node", "js/tools/samples.js"], cwd=ROOT, text=True))
    toggle(samples)
    stagger(samples)
    counter(samples)
    parity(samples)


if __name__ == "__main__":
    main()
