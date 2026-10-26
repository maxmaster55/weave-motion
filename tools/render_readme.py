#!/usr/bin/env python3
"""Render GitHub-friendly GIFs from real Weave samples.

From the repo root: nix develop .#docs --command python tools/render_readme.py
These are documentation illustrations, not GPU/browser recordings.
"""
import csv
import io
import os
from pathlib import Path
import subprocess

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
FONT = os.environ["WEAVE_DOCS_FONT"]
BG, CARD, LINE, WHITE, MUTED = "#0c1018", "#141b28", "#283449", "#f1f4fa", "#8e9caf"
COLORS = ["#a5b4fc", "#79dfc1", "#f6c78d", "#e4a6e9"]
SCALE = 2


def font(size):
    return ImageFont.truetype(FONT, size * SCALE)


def render(kind, rows):
    frames = []
    titles = ["A button becomes a surface.", "Four ways to get there.", "Change your mind. Keep moving."]
    subtitles = ["Shared identity / spring bounds / rounded clip", "The same destination, different spring presets", "Retarget mid-flight without resetting velocity"]
    for i, row in enumerate(rows):
        image = Image.new("RGB", (800 * SCALE, 400 * SCALE), BG)
        draw = ImageDraw.Draw(image)

        def text(x, y, value, size=14, fill=WHITE):
            draw.text((x*SCALE, y*SCALE), value, font=font(size), fill=fill)

        def line(xy, fill=LINE, width=1):
            draw.line(tuple(v*SCALE for v in xy), fill=fill, width=width*SCALE)

        def box(xy, fill, radius=0, outline=None):
            draw.rounded_rectangle(tuple(v*SCALE for v in xy), radius=radius*SCALE,
                                   fill=fill, outline=outline, width=SCALE)

        text(32, 23, "weave", 18)
        text(600, 28, ["01 / SHAPE + SCENE", "02 / SPRING TOKENS", "03 / INTERRUPTION"][kind], 11, MUTED)
        line((32, 61, 768, 61))
        text(32, 78, titles[kind], 25)
        text(32, 118, subtitles[kind], 13, MUTED)
        if kind == 0:
            x, y, w, h, r = row[8:13]
            # Center the example's 720-wide coordinate space in the card.
            x += 40
            y = y * .73 + 151
            h *= .73
            box((x, y, x+w, y+h), "#b6c1ff", min(r, h/2))
            if w > 320:
                text(x+25, y+20, "A little more room.", 21, BG)
                text(x+25, y+58, "The surface keeps its identity.", 13, "#343e61")
                box((x+25, y+91, x+w-25, y+95), "#8a99da", 2)
            else:
                text(x+w/2-27, y+h/2-10, "+ Open", 14, BG)
            state = "EXPAND" if 15 <= i < 100 else "COLLAPSE" if i >= 100 else "AT REST"
            text(32, 357, state, 11, COLORS[0])
            text(470, 357, "Actual weave-scene bounds and clip", 11, MUTED)
        elif kind == 1:
            names = ["fast_spatial", "default_spatial", "expressive_spatial", "gentle_spatial"]
            for n, name in enumerate(names):
                y = 181 + n*43
                text(32, y-10, name, 13, MUTED)
                line((238, y, 735, y), LINE, 2)
                line((700, y-9, 700, y+9), "#48556c")
                x = 250 + row[n+1]*450
                box((x-7, y-7, x+7, y+7), COLORS[n], 7)
            text(32, 368, "EXACT-TIME SPRINGS", 11, COLORS[1])
            text(535, 368, "One shared 30 Hz sample clock", 11, MUTED)
        else:
            left, right, top, bottom = 38, 760, 169, 324
            for fraction in [0, .5, 1]:
                y = bottom - fraction*(bottom-top)
                line((left, y, right, y))
            # Target trace and physical trajectory use the same time axis.
            for j in range(1, i+1):
                x0, x1 = left+(j-1)/179*(right-left), left+j/179*(right-left)
                line((x0, bottom-rows[j-1][6]*135, x1, bottom-rows[j][6]*135), "#4b566a", 1)
                line((x0, bottom-rows[j-1][5]*135, x1, bottom-rows[j][5]*135), COLORS[2], 3)
            x, y = left+i/179*(right-left), bottom-row[5]*135
            box((x-5, y-5, x+5, y+5), WHITE, 5)
            text(32, 357, "POSITION + VELOCITY PRESERVED", 11, COLORS[2])
            text(574, 357, "Dim line = destination", 11, MUTED)
        image = image.resize((800, 400), Image.Resampling.LANCZOS)
        frames.append(image.quantize(colors=96))
    output = ROOT / "docs/assets" / ["pill-to-panel.gif", "spring-presets.gif", "retargeting.gif"][kind]
    frames[0].save(output, save_all=True, append_images=frames[1:],
                   duration=[30 if i % 3 else 40 for i in range(len(frames))], loop=0, optimize=True, disposal=2)
    print(f"{output.relative_to(ROOT)}: {output.stat().st_size:,} bytes")


def main():
    samples = subprocess.check_output(
        ["cargo", "run", "--quiet", "-p", "weave-scene", "--example", "readme_samples"], cwd=ROOT, text=True)
    rows = [[float(v) for v in row] for row in csv.reader(io.StringIO(samples))]
    assert len(rows) == 180 and all(len(row) == 13 for row in rows)
    for kind in range(3):
        render(kind, rows)


if __name__ == "__main__":
    main()
