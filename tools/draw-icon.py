#!/usr/bin/env python3
"""Draw the app icon ([050], D31): the title screen's RIX box, in the game font, on dark.

The box has the logo's purple fill and a glowing pale edge; the letters glow a little;
the colours split slightly left and right, and faint scanlines run across, as the game's
CRT pass does.

Writes into the mobile projects:
- iOS: the 1024 px icon of the asset catalog, a full square with no alpha (iOS rounds
  the corners itself).
- Android: the adaptive icon for each density. A launcher shows the middle 72 of the
  108 dp layer through its mask, and every mask keeps the middle 66 dp circle, so the
  box's corners sit inside that circle. The foreground is the box with its glow over a
  clear layer (its colours are worked out so that over the background it looks as the iOS
  icon does), the background is the dark colour, and the monochrome layer (themed icons)
  is the box's edge and the letters.

Usage: tools/draw-icon.py
"""
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parent.parent
FONT = ROOT / "assets/fonts/main.ttf"
IOS = ROOT / "mobile/ios/ios-src/Assets.xcassets/AppIcon.appiconset/icon-1024.png"
ANDROID_RES = ROOT / "mobile/android/app/src/main/res"
# Android densities and the pixel size of a 108 dp layer there.
DENSITIES = {"mdpi": 108, "hdpi": 162, "xhdpi": 216, "xxhdpi": 324, "xxxhdpi": 432}

VOID = (21, 24, 26)
PURPLE = (78, 52, 125)
EDGE = (255, 225, 240)
LETTERS = (240, 244, 236)

MASTER = 1024
# The box on the 1024 master: 724 x 500, centred.
BOX = (724, 500)
# In the 108 dp Android layer, the box's diagonal fills this share of the 66 dp circle.
SAFE_SHARE = 0.97


def neon(img, layer, radius, gain):
    """Adds the layer's blurred glow to img, then lays the sharp layer on top."""
    blur = layer.filter(ImageFilter.GaussianBlur(radius))
    blur = Image.eval(blur, lambda v: min(255, int(v * gain)))
    img = ImageChops.add(img, blur)
    mask = layer.convert("L").point(lambda v: 255 if v else 0)
    return Image.composite(layer, img, mask)


def scanlines(img, period, dark=0.92):
    mask = Image.new("L", img.size, 255)
    d = ImageDraw.Draw(mask)
    for y in range(0, img.size[1], period):
        d.rectangle([0, y + period // 2, img.size[0], y + period - 1], fill=int(255 * dark))
    return ImageChops.multiply(img, Image.merge("RGB", [mask] * 3))


def chroma(img, shift):
    r, g, b = img.split()
    return Image.merge("RGB", (ImageChops.offset(r, -shift, 0), g, ImageChops.offset(b, shift, 0)))


def shapes(size, scale):
    """The box and the letters as masks: the fill, the edge and the text."""
    w, h = BOX[0] * scale, BOX[1] * scale
    box = [(size - w) / 2, (size - h) / 2, (size + w) / 2, (size + h) / 2]
    radius, edge = 44 * scale, max(1, round(14 * scale))
    fill = Image.new("L", (size, size), 0)
    ImageDraw.Draw(fill).rounded_rectangle(box, radius, fill=255)
    rim = Image.new("L", (size, size), 0)
    ImageDraw.Draw(rim).rounded_rectangle(box, radius, outline=255, width=edge)
    text = Image.new("L", (size, size), 0)
    td = ImageDraw.Draw(text)
    font = ImageFont.truetype(str(FONT), round(330 * scale))
    l, t, r, b = td.textbbox((0, 0), "RIX", font=font)
    td.text(((size - (r - l)) / 2 - l, (size - (b - t)) / 2 - t), "RIX", font=font, fill=255)
    return fill, rim, text


def draw(size, scale):
    """The icon on the dark ground: `scale` is the box's size against the 1024 master."""
    fill, rim, text = shapes(size, scale)
    img = Image.new("RGB", (size, size), VOID)
    img.paste(PURPLE, mask=fill)
    colour = lambda mask, rgb: Image.composite(Image.new("RGB", (size, size), rgb),
                                               Image.new("RGB", (size, size)), mask)
    img = neon(img, colour(rim, EDGE), 26 * scale, 0.9)
    img = neon(img, colour(text, LETTERS), 18 * scale, 0.45)
    return scanlines(chroma(img, max(1, round(4 * scale))), max(2, round(8 * scale)))


def over_void(img):
    """A clear layer that looks like `img` when laid over the dark ground."""
    out = Image.new("RGBA", img.size)
    px, dst = img.load(), out.load()
    for y in range(img.size[1]):
        for x in range(img.size[0]):
            c = px[x, y]
            a = max(max(0, c[i] - VOID[i]) / (255 - VOID[i]) for i in range(3))
            if a <= 0:
                continue
            f = tuple(min(255, round((c[i] - VOID[i] * (1 - a)) / a)) for i in range(3))
            dst[x, y] = f + (round(a * 255),)
    return out


def android(size):
    # The box's diagonal against the 66 dp circle of a 108 dp layer.
    diagonal = (BOX[0] ** 2 + BOX[1] ** 2) ** 0.5
    scale = SAFE_SHARE * size * 66 / 108 / diagonal
    foreground = over_void(draw(size, scale))
    _, rim, text = shapes(size, scale)
    mono = Image.new("RGBA", (size, size), (255, 255, 255, 0))
    mono.putalpha(ImageChops.lighter(rim, text))
    return foreground, mono


def main():
    IOS.parent.mkdir(parents=True, exist_ok=True)
    draw(MASTER, 1.0).save(IOS)
    for density, size in DENSITIES.items():
        out = ANDROID_RES / f"mipmap-{density}"
        out.mkdir(parents=True, exist_ok=True)
        foreground, mono = android(size)
        foreground.save(out / "ic_launcher_foreground.png")
        mono.save(out / "ic_launcher_monochrome.png")
    print(f"{IOS.relative_to(ROOT)} and {len(DENSITIES)} Android densities")


if __name__ == "__main__":
    main()
