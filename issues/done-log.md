# Done log

Every closed issue, newest first, with a one-line summary. History: read it on demand,
never at session start. The issue file in `done/` holds the full record. When an issue
closes, add its line at the top, in the commit that moves its file to `done/`.

- [077] M3: Input in event order - `WindowEvent` steps (press, move, release) run the line logic one by one; a tick's line messages merge, so no segment stays; harness writes both touch streams
- [032] M3: Data-driven enemy spawning - `EnemyKind` spawn table with level bands and weights, `run_system_cached` (no `EnemySystems`/`ItemSystems`), spawn search bounded to 64 tries; shots unchanged
- [072] M3: Same seed, same run - gameplay in `FixedUpdate` on one thread, avian `enhanced-determinism`, `just replay` (5 seeds at 30/60/120 fps)
- [065] M3: No mass or inertia - arrow gets inertia and locked rotation, bomb loses `Mass(0.)`; 0 warnings in 12 seeds, shots unchanged
- [063] M3: Sleeping bodies and the line - not reproduced on avian 0.8 (standalone collider tree); regression test added
- [031] M3: Score overflow - `level_points` saturates; the score stops at `u128::MAX`
- [030] M3: Kill count - kills count once and explode in the kill observer; enemies that leave with the state are no kill
- [074] M3: Territory model A - `FreeRegion` cut by `split`, input rule `line_end`, exact progress; splice code gone; fuzz test 2,000 claims 0 invalid
- [028] M3: Split LevelInfo into `Run`, `LevelProgress` and `GameRng`; the HUD redraws only when run or progress change
- [027] M3: Asset loading - `asset_tracking` (`LoadResource`), `Screen::Loading` replaces `Setup`, `FontAssets` and `ItemAssets`; no system loads by path; shot runs count from the title so loading time does not move them
- [029] M3: Spike - territory model: free-region polygon cut (A) 0 invalid in 160k fuzzed claims, 3-35 µs per cut, vs grid (B) about 1 ms and stepped edges; Pedro chose A (D19), build in [074]
- [026] M3: bevy_new_2d structure - `Screen`/`Phase` states with `DespawnOnExit`, `PausableSystems` and `AppSystems` sets (arrow seeds now replay), `screens/` `game/` `theme/` layout with `pub(super) fn plugin`, release builds drop `info!`; `just shot-flow` entity census; shots byte-identical per move
- [024] M2: Mobile performance pass - claim collider is a trimesh of the lyon fill (was VHACD, 4-16 ms), area is an exact scanline union (was 100k-sample Monte Carlo, 67 ms late-level); claim frames back to normal on emulator and simulator; dev_tools `slow update` log
- [023] M2: Mobile platform behaviour - letterbox clears the notch, pause on suspend ([034]), Android Back moves to background, default frame pacing; suspend/resume checked on emulator and simulator
- [034] M3 (pulled into M2 for [023]): Pause - suspend, focus loss or Escape pauses (virtual + physics time), a tap resumes, a line in progress is cancelled; button and menu split to [073]
- [019] M2: Resolution-independent playfield - fixed 450x800 field (D17), AutoMin letterbox camera, input through the camera clamped to the field, HUD as world Text2d, RIX_SHOT_SIZE; same claim at 4 sizes, rotation mid-run on Android
- [020] M2: iOS build on main - mobile/ios Xcode project from Bevy examples/mobile (builds the rix bin), just ios-sim / ios-device, iOS 15.0, top.cosas.rix; runs on the iPhone 16 simulator
- [022] M2: Android crash at start - cause: emulator Vulkan driver segfaults on wgpu debug object names; fix: no InstanceFlags::DEBUG on Android; 2 min play loop on the API 36 emulator, no crash
- [021] M2: Android build on main - mobile/ crate (cdylib, bevy_main), Gradle 9.4 + AGP 9.1 + games-activity 4.4.0 project, just android; APK installs on the API 36 emulator
- [008] M0: CI on GitHub Actions - fmt, clippy -D warnings, tests on ubuntu in a ci profile; first run green; closes M0 and M1 (version 0.3.0)
- [066] M0: rustfmt and clippy clean - cargo fmt, 7 collapsible ifs as let-chains, 3 unwraps after is_some; shots identical
- [009] M0: Repo hygiene - gitignore for mobile outputs and .DS_Store; feat/touch and feat/splitter deleted; the rest go with [020], [021], [040] (D11)
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
