# [013] Replace bevy_prototype_lyon with an own lyon_tessellation mesh module

**Status:** open
**Milestone:** M1
**Depends on:** [006], [007]
**Created:** 2026-11-02
**Closed:** -

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
