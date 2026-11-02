# [013] Replace bevy_prototype_lyon with an own lyon_tessellation mesh module

**Status:** done
**Milestone:** M1
**Depends on:** [006], [007]
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
`bevy_prototype_lyon` is pulled from its git `master` branch (locked at f5a3610, 0.13.0).
Master now targets Bevy 0.19, so a plain `cargo update` breaks the build, and no release
exists for Bevy 0.20 (checked 2026-11-02). The game also uses the render component as its
geometry store: `extract_points_from_shape` flattens the lyon `Shape` path back into points
in `polygon.rs`, `level.rs` and `user_input.rs`.

## Approach
(D2) Add `src/shape.rs`: build a `Mesh` from a closed polygon (`lyon_tessellation`
`FillTessellator`) and a stroke outline (`StrokeTessellator`), drawn as `Mesh2d` +
`MeshMaterial2d<ColorMaterial>` (stroke as a child entity). Territory polygons keep their
points in a component (`Territory` with the vertex list), and every reader uses that
instead of flattening a `Shape`. Fades change the `ColorMaterial` alpha. Replace the three
uses: claimed polygons (fill), the Arrow enemy (rounded polygon fill + stroke, stroke
width changes while drawing), the menu box (rounded rectangle stroke + fill). Remove the
dependency.

## Verification
`cargo tree -i bevy_prototype_lyon` finds nothing; `cargo test` passes; `just shot` shows a
claimed area as before (compare with the pre-change shot); `just shot-menu` shows the box.

## Notes
- Design change from the Approach: `src/shape.rs` keeps a small `Shape` component (lyon
  `Path`, optional fill color, optional stroke color and width) and re-meshes it on
  `Changed<Shape>` in `PostUpdate`, as the old plugin did: one mesh with vertex colors
  (fill first, stroke on top) and one shared white `ColorMaterial`. This keeps the look
  (same lyon tessellator, same default fill and stroke options) and keeps the call sites
  (fade, arrow stroke width, menu stroke flicker) almost unchanged. A stroke child entity
  and per-entity material alpha would change draw order and blending for no gain.
- Same lyon versions as the old lock (`lyon_tessellation` 1.0.15, `lyon_algorithms`
  1.0.5): `lyon_algorithms::rounded_polygon` rounds the arrow, as before.
- `Territory(Vec<Vec2>)` on each claimed polygon holds its world outline (the
  `DrawPolygon` path, last point = first point). `user_input.rs` and `level.rs` read it.
  `extract_points_from_shape` is gone; it returned the same points plus one more copy of
  the first point, which changes no result (zero-length edge; nearest-vertex search keeps
  the first of equal points).
- Side effect, a bug fix: `calculate_area_progress` queried every lyon `Shape`, so each
  Arrow enemy's local outline (around the world origin) counted as claimed area, and each
  arrow spawn re-ran the 100,000-sample estimate on the level RNG. Now only `Territory`
  counts. Seeded runs with arrows draw different random numbers after this change.
- Done. `cargo tree -i bevy_prototype_lyon`: no match. `cargo test`: 32 pass (4 new in
  `shape.rs`: fill triangles and colors, stroke after fill, empty shape, rounded shapes stay
  in their bounds). Clippy: no new warnings (15, all pre-existing).
- Shots against the Bevy 0.15 lyon baseline, seed 1: game 14 px and menu 8 px differ above
  3 % fuzz out of 1,562,500 (PSNR 57.8 and 56.0 dB; two baseline runs: 62.6 dB). Points 9,
  as before. Arrow (seed 4, old build from HEAD in a temporary worktree vs new): same
  rounded outline, fill and stroke. Seed 4 runs are not deterministic on either build
  (frame timing): filed [062].
