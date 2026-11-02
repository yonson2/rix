# Done log

Every closed issue, newest first, with a one-line summary. History: read it on demand,
never at session start. The issue file in `done/` holds the full record. When an issue
closes, add its line at the top, in the commit that moves its file to `done/`.

- [005] Infra: Issue board Vercel deploy - dropped (D10: the board stays local)
- [012] Gate: v1 content scope - answered: Splitter and Shooter, all three items, endless with one death, show and share seeds per run, Game Center / Play Games leaderboards; filed [067], [068]
- [011] Gate: Art and audio direction - answered: keep neon + CRT (refs Geometry Wars, Hyper Light Drifter), font OFL ships, own music and SFX, procedural or PixelLab images, new icon
- [061] M6: GOG release - dropped (D9: no GOG)
- [060] M6: Steam release - dropped (D9: no Steam)
- [010] Gate: Product decisions for v1 - answered: Rix final, top.cosas.rix, free no ads, both orientations, fixed playfield, web on itch.io, no Steam/GOG (D9), English
- [002] M0: ARCHITECTURE.md - code map, state machine, line data flow, cross-cutting concerns, invariants
- [064] M0: Headless shots - no winit in a shot run, schedule runner, no macOS focus code or objc2 deps; shots identical
- [018] M1: Bevy 0.20.1 + avian2d 0.8.0 - WESL shaders, extract_app, On<Remove<T>>; all five shaders checked (headless probe); shots match
- [017] M1: Bevy 0.19.1 + avian2d 0.7.0 - crt pass as a Core2d system, TextFont/AssetMut; shots render offscreen at 625 (hidden window and scale-2 image draw nothing)
- [016] M1: Bevy 0.18.1 + avian2d 0.6.1 - bind group layout descriptor in crt; same-state transitions checked; shots match
- [015] M1: Bevy 0.17.3 + avian2d 0.4.1 - events to messages, observers to `On`, crt pipeline in `RenderStartup`, `CollisionStart`; shots match
- [014] M1: Bevy 0.16.1 + avian2d 0.3.1 - mechanical renames, CollisionStarted with opt-in events; shots match
- [062] M0: Deterministic shots - fixed frame time, item spawn after enemies (seed bug), macOS focus give-back
- [001] M0: AGENTS.md - the game, goal, tracker discipline, hard rules, git, commands
- [004] M0: Issue board - mud's Astro board in `tracker/`, Rix neon look, milestone goals; 39 tests
- [013] M1: Replace bevy_prototype_lyon - own `src/shape.rs` (lyon fill and stroke mesh), `Territory` outlines; shots match the baseline
- [007] M0: Unit tests - 28 tests for geometry, input, walls, score display, enemy count; quirks pinned
- [006] M0: Dev harness - RIX_SEED, RIX_SHOT, just shot and shot-menu; objc2 debug checks off for macOS 27
- [003] M0: Issue tracker - mud format, 61 issues across M0-M6, 3 gates; todo.md migrated and removed
