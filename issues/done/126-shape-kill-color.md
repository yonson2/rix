# [126] A shape enemy's kill is white and brightens every shape

**Status:** done
**Milestone:** M8
**Depends on:** [039], [116]
**Created:** 2026-11-05
**Closed:** 2026-11-05

## Problem
`on_despawn_enemy` takes the kill color from the enemy's `ColorMaterial` before its
`Shape` fill. A shape enemy (arrow, splitter, shooter) has the shared white shape
material (`ShapeMaterial`), so its sparks and its SHOCKWAVE rings ([116]) are white, not
the enemy's color. Its corpse also gets that shared material, and `animate_corpses` sets
the material's color: during the corpse's white pop, every shape on screen (claims, other
shape enemies) is drawn at `NEON_WHITE`, twice as bright, for about 0.1 s.

## Approach
- The kill color: the shape's fill first, then the material's color.
- A shape enemy's corpse is a copy of its `Shape` (its own mesh, the shared material
  unchanged); `animate_corpses` sets the shape's fill, as it does now for a shape corpse.
- A test: a shape enemy's kill leaves the shared material white and gives its corpse
  and burst the enemy's fill.

## Verification
`cargo test` passes with that test; `RIX_SHOT_SCENE=kill RIX_SKINS=shockwave` (seed 14,
level 6) shows the rings in the arrows' and splitters' colors; `just replay` unchanged.

## Notes
- Found 2026-11-05 in [116]: the SHOCKWAVE rings of arrows and splitters were white.
- Done 2026-11-05. `on_despawn_enemy` takes the shape's fill first, and a shape enemy's
  corpse copies its `Shape`. `animate_corpses` sets a shape corpse's fill and leaves its
  material alone: `mesh_shapes` gives the new corpse the shared material, so the first fix
  alone tinted every shape in the arrow's yellow while the corpse lived.
- The sparks of a shape enemy's kill (`Burst::kill`) are now in its color too.
- Evidence: `cargo test` 151 passed (`a_shape_enemy_dies_in_its_fill_and_leaves_the_shared_material_white`
  gives the corpse the shared material, as `mesh_shapes` does); clippy clean with and
  without `dev_tools`; `just replay` unchanged; `just shot` md5
  e170997305a806320c6f9f967cf3cf5b (unchanged); kill shots (seed 14, level 6) with each
  kill skin: purple rings for splitters, yellow for arrows, the arrow corpse yellow.
