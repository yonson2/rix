# [099] Shader origins: the flame shader may be a Balatro port

**Status:** done
**Milestone:** M5
**Depends on:** [052]
**Created:** 2026-11-03
**Closed:** 2026-11-03

## Problem
Every file under `assets/` ships in both apps (Android `assets.srcDirs`, the iOS
`assets` folder reference). `assets/shaders/flame_material.wesl` has the uniforms of
Balatro's flame shader (`time`, `amount`, two skipped bindings where Balatro has
`texture_details` and `image_details`, then `colour_1`, `colour_2`). Balatro's code is not
open source. Only `examples/flame_shader.rs` uses it; the game does not.

The other shaders (`crt_shader`, `glow_material`, `cartoon_explosion`,
`pixel_explosion`) date from April 2025, before the tracker; their origin is not recorded.

## Approach
Pedro states the origin of each shader. A shader of unknown or non-open origin leaves
`assets/` (delete it with its example, or move both out of the shipped folder). Record
the answer in `CREDITS.md`.

## Verification
`CREDITS.md` names the origin of every file in `assets/shaders/`; no shipped shader has an
unknown origin; `cargo build --examples` and `just shot` pass.

## Notes
- Found in [052] (2026-11-03).
- **owner** answer needed: where each shader came from.
- Pedro (2026-11-03, D34): "Delete unused shaders, otherwise they have been programmed
  from scratch looking at other references". The game uses four shaders (`crt_shader`,
  `cartoon_explosion`, `pixel_explosion`, `glow_material`); `flame_material.wesl` and
  `examples/flame_shader.rs` are deleted. `CREDITS.md` gives the origin. `cargo build
  --examples` passes; `just shot` gives md5 e170997305a806320c6f9f967cf3cf5b.
