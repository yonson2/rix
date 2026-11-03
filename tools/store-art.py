#!/usr/bin/env python3
"""Make the store art ([051], [101]) in target/store-art: the screenshots come from the
game itself through the dev shot harness (`RIX_SHOT`), each under a caption, then the
Play feature graphic (the logo box between two of the Play shots) and the Play icon
(`tools/draw-icon.py`).

Each game shot is a seed and a level in the store scene (three lines, the last one half
drawn), so after a visual change one run makes the set again. The field takes the image
shape, so each size has its own seeds: seeds whose lines claim two parts of the field and
leave enemies. The raw shots stay in target/store-art/raw.

Sizes (read 2026-11-03):
- App Store: iPhone 1206 x 2622; Rix is iPhone only (D32), so no iPad set.
  https://developer.apple.com/help/app-store-connect/reference/screenshot-specifications/
- Play: phones 1080 x 2160 (the long side at most twice the short one), feature graphic
  1024 x 500, icon 512 x 512 (32-bit PNG under 1 MB).
  https://support.google.com/googleplay/android-developer/answer/9866151
The screenshots and the feature graphic have no alpha.

Usage: just store-art (it builds the game with dev_tools first)
"""
import importlib.util
import os
import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageEnhance, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "target/store-art"
BIN = ROOT / "target/debug/rix"
FONT = ROOT / "assets/fonts/main.ttf"

SIZES = {
    "app-store/iphone": (1206, 2622),
    "play/phone": (1080, 2160),
}
# Per size: (seed, level, skins) for each game shot. Seed 17 has the STAR power on, seed 4
# the x2 gem.
PHONE = [(17, 6, None), (4, 9, "laser,brick"), (19, 9, "chain,circuit")]
GAME = {
    "app-store/iphone": PHONE,
    "play/phone": PHONE,
}
# No title shot: the shot build is a desktop build, and its title has a QUIT button that
# phones do not have.
MENUS = ["wiki"]
# One caption per shot, in shot order.
CAPTIONS = [
    "DRAW A LINE\nCLAIM THE FIELD",
    "GRAB POWER-UPS\nDODGE THE SWARM",
    "UNLOCK NEW\nLINES AND WALLS",
    "KNOW YOUR\nENEMIES",
]
# The caption band's share of the height, and the frame's gap to the sides and the bottom
# as a share of the width.
CAPTION_SHARE = 0.15
FRAME_GAP = 0.08
FEATURE = (1024, 500)
# The logo box against the icon's: 449 x 310 px, in the middle.
LOGO_SCALE = 0.62
# The Play shots at the sides: their brightness and their gap to the edge.
SIDE_LIGHT = 0.5
SIDE_GAP = 12


def shot(path, size, scene, seed=1, level=None, skins=None):
    env = dict(os.environ, BEVY_ASSET_ROOT=str(ROOT), RIX_SHOT=str(path), RIX_SHOT_SCENE=scene,
               RIX_SHOT_SIZE=f"{size[0]}x{size[1]}", RIX_SEED=str(seed))
    if level:
        env["RIX_LEVEL"] = str(level)
    if skins:
        env["RIX_SKINS"] = skins
    subprocess.run([BIN], env=env, check=True, capture_output=True, timeout=30)
    img = Image.open(path).convert("RGB")
    assert img.size == size, (path, img.size)
    img.save(path)
    return img


def draw_icon():
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location("draw_icon", ROOT / "tools/draw-icon.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def colour(mask, rgb):
    return Image.composite(Image.new("RGB", mask.size, rgb), Image.new("RGB", mask.size), mask)


def captioned(raw, caption, icon):
    """The shot under its caption: glowing letters in the game font on the dark ground,
    the shot scaled into a frame with the logo box's glowing edge."""
    w, h = raw.size
    band = round(h * CAPTION_SHARE)
    gap = round(w * FRAME_GAP)
    scale = min((w - 2 * gap) / w, (h - band - gap) / h)
    inner = raw.resize((round(w * scale), round(h * scale)), Image.LANCZOS)
    x, y = (w - inner.width) // 2, band
    radius, edge = round(w * 0.03), max(2, round(w * 0.006))

    img = Image.new("RGB", (w, h), icon.VOID)
    corners = Image.new("L", inner.size, 0)
    ImageDraw.Draw(corners).rounded_rectangle([0, 0, *inner.size], radius, fill=255)
    img.paste(inner, (x, y), corners)
    rim = Image.new("L", (w, h), 0)
    ImageDraw.Draw(rim).rounded_rectangle(
        [x - edge, y - edge, x + inner.width + edge, y + inner.height + edge], radius + edge,
        outline=255, width=edge)
    img = icon.neon(img, colour(rim, icon.EDGE), w * 0.012, 0.9)

    # One letter size for every caption: the longest line fills the frame's width.
    size = round(band * 0.3)
    widest = max(ImageFont.truetype(str(FONT), size).getlength(line)
                 for c in CAPTIONS for line in c.split("\n"))
    font = ImageFont.truetype(str(FONT), min(size, int(size * (w - 2 * gap) / widest)))
    text = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(text)
    draw.multiline_text((w / 2, band / 2), caption, font=font, fill=255, anchor="mm",
                        align="center", spacing=round(font.size * 0.25))
    return icon.neon(img, colour(text, icon.LETTERS), w * 0.008, 0.6)


def main():
    icon = draw_icon()
    for name, size in SIZES.items():
        raw, out = OUT / "raw" / name, OUT / name
        raw.mkdir(parents=True, exist_ok=True)
        out.mkdir(parents=True, exist_ok=True)
        shots = [(f"{i}-game", "store", seed, level, skins)
                 for i, (seed, level, skins) in enumerate(GAME[name], 1)]
        shots += [(f"{i}-{scene}", scene, 1, None, None)
                  for i, scene in enumerate(MENUS, len(GAME[name]) + 1)]
        assert len(shots) == len(CAPTIONS)
        for (file, scene, seed, level, skins), caption in zip(shots, CAPTIONS):
            img = shot(raw / f"{file}.png", size, scene, seed, level, skins)
            captioned(img, caption, icon).save(out / f"{file}.png")
    play = OUT / "play"
    feature = Image.new("RGB", FEATURE, icon.VOID)
    for i in (1, 2):
        side = Image.open(OUT / f"raw/play/phone/{i}-game.png")
        side = side.resize((side.width * FEATURE[1] // side.height, FEATURE[1]), Image.LANCZOS)
        side = ImageEnhance.Brightness(side).enhance(SIDE_LIGHT)
        feature.paste(side, (SIDE_GAP if i == 1 else FEATURE[0] - SIDE_GAP - side.width, 0))
    # The logo box in the middle: its glow over the ground, the box itself solid.
    logo = icon.draw(FEATURE[1], LOGO_SCALE)
    fill, _, _ = icon.shapes(FEATURE[1], LOGO_SCALE)
    x = (FEATURE[0] - FEATURE[1]) // 2
    glow = Image.new("RGBA", FEATURE)
    glow.paste(icon.over_void(logo), (x, 0))
    feature = Image.alpha_composite(feature.convert("RGBA"), glow).convert("RGB")
    feature.paste(logo, (x, 0), fill)
    feature.save(play / "feature-graphic.png")
    icon.draw(512, 0.5).convert("RGBA").save(play / "icon-512.png")
    assert (play / "icon-512.png").stat().st_size < 1_000_000
    print(f"{OUT.relative_to(ROOT)}: " + ", ".join(sorted(
        str(p.relative_to(OUT)) for p in OUT.rglob("*.png") if "raw" not in p.parts)))


if __name__ == "__main__":
    main()
