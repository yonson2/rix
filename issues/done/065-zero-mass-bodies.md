# [065] Dynamic bodies with no mass or inertia (bomb, arrow)

**Status:** done
**Milestone:** M3
**Depends on:** -
**Created:** 2026-11-02
**Closed:** 2026-11-02

## Problem
Found during [064]: a seed 10 shot run logs
`WARN avian2d::dynamics::rigid_body::mass_properties: Dynamic rigid body 606v0 has no mass
or inertia. This can cause NaN values.` Two spawns can cause it (not yet told apart):
- `items/basic_bomb.rs`: `RigidBody::Dynamic` with `Mass(0.)`.
- `enemies/arrow.rs`: `Collider::polyline` has no area, so its angular inertia is zero
  (`Mass(3.0)` sets the mass only).
Earlier shot logs were filtered, so it is not known since which avian version it shows.

## Approach
Find which entity logs it (`Name` on the arrow, or a spawn log). Give the body real mass
properties: a bomb mass above zero (or a `Sensor` / kinematic body if it must not push),
and for the arrow an `AngularInertia` or a closed collider (`Collider::convex_hull` of the
points), or lock rotation if `ArrowRotation` drives it. Check that it plays the same.

## Verification
`RIX_SEED=10 RIX_SHOT=target/shots/065.png cargo run --features dev_tools 2>&1 | grep -c
"no mass or inertia"` prints 0 (and the same for a few other seeds with a bomb or arrow);
`just shot` matches the previous shot.

## Notes
- Done (2026-11-02): both spawns caused it. avian stores the inverse: a zero mass or
  inertia becomes `INFINITY`, which `warn_invalid_mass` reports.
  - Arrow (seeds 6, 7, 10, 11): the polyline gives no inertia. It now has
    `AngularInertia(1.)` and `LockedAxes::ROTATION_LOCKED`; contacts never turned it
    before either (infinite inertia), and `point_arrow_to_cursor` still sets its rotation.
  - Bomb (seeds 2, 5, 9, 12): `Mass(0.)` is gone, so the 40 x 40 collider gives the mass
    (density 1). It has no velocity and only meets walls, claims and the line, so it
    stays put as before.
- Evidence: `RIX_SEED=<s> RIX_SHOT=... target/debug/rix 2>&1 | grep -c 'no mass or
  inertia'` was 1 for 8 of seeds 1-12 and is now 0 for all 12; all 12 shots are
  byte-identical to the shots before the change (seed 1 `109eb09e`). `cargo test` 38
  passed (the [063] test's bomb bundle follows the spawn), clippy clean both ways, fmt
  clean.
