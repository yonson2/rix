#!/usr/bin/env python3
"""Draw the bomb item's image ([098]): a round blue bomb with a cap and a striped fuse, on
a clear 100 x 100 canvas, as the old image of unknown source was. Drawn 4 times larger,
then scaled down, for smooth edges.

Usage: tools/draw-bomb.py OUT.png   (the game's file: assets/items/bomb_basic.png)
"""
import math
import sys

from PIL import Image, ImageDraw

SIZE = 100
SCALE = 4
BODY = (89, 127, 232, 255)
SHADE = (70, 102, 200, 255)
SHINE = (150, 180, 250, 255)
INK = (24, 32, 60, 255)
FUSE = (255, 253, 231, 255)
STRIPE = (200, 190, 160, 255)


def s(v):
    return v * SCALE


def main():
    img = Image.new("RGBA", (s(SIZE), s(SIZE)), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx, cy, r = 47, 63, 28
    # The fuse: a curve from the cap upward, drawn as short striped pieces.
    points = [(71 - 5 * math.sin(t / 9 * math.pi), 36 - t * 3.0) for t in range(10)]
    for i, (a, b) in enumerate(zip(points, points[1:])):
        d.line([s(a[0]), s(a[1]), s(b[0]), s(b[1])], fill=INK, width=s(10))
    for i, (a, b) in enumerate(zip(points, points[1:])):
        d.line([s(a[0]), s(a[1]), s(b[0]), s(b[1])], fill=STRIPE if i % 2 else FUSE, width=s(7))
    tip = points[-1]
    for dx, dy in ((-4, -6), (0, -7), (4, -5)):
        d.line([s(tip[0]), s(tip[1]), s(tip[0] + dx), s(tip[1] + dy)], fill=INK, width=s(2))
    # The cap: a short square neck at 45 degrees, between the body and the fuse.
    angle = math.radians(-45)
    ux, uy = math.cos(angle), math.sin(angle)
    vx, vy = -uy, ux
    base = (cx + ux * (r - 4), cy + uy * (r - 4))
    corners = [(base[0] + ux * a + vx * b, base[1] + uy * a + vy * b)
               for a, b in ((0, -10), (11, -10), (11, 10), (0, 10))]
    d.polygon([(s(x), s(y)) for x, y in corners], fill=SHADE, outline=INK, width=s(2))
    # The body and its shine.
    d.ellipse([s(cx - r), s(cy - r), s(cx + r), s(cy + r)], fill=BODY, outline=INK, width=s(2))
    d.ellipse([s(cx - 16), s(cy - 18), s(cx - 6), s(cy - 9)], fill=SHINE)
    img.resize((SIZE, SIZE), Image.LANCZOS).save(sys.argv[1])


if __name__ == "__main__":
    main()
