# Splitter Enemy - Specification

> **Built in [044] (2026-11-03)**, adapted to Bevy 0.20: `src/game/enemies/splitter.rs`.
> Changes from this sketch: a plugin function, not `SplitterPlugin`; the Queen's death
> spawns a `Hatch` entity (no event) that becomes the Soldiers 0.25 s later; both kinds are
> `Shape` outlines (octagon, ellipse) in purple; the Splitter spawns from level 2. Step 8
> was [030].

**Purpose:** This document outlines the design and implementation details for the "Splitter" enemy type in the Rix game.

**Concept:** The Splitter enemy starts as a single entity (the "Queen"). When the area containing the Queen is claimed by the player, instead of simply disappearing, the Queen triggers the spawning of multiple smaller, faster enemies (the "Soldiers"). This adds a dynamic challenge, forcing the player to deal with the consequences of capturing certain enemies.

---

- [ ] **1. Define `SplitterType` Component:**
  - [ ] Create `src/game/enemies/splitter.rs`.
  - [ ] Define a public enum `SplitterType { Queen, Soldier }`.
  - [ ] This enum itself will be used as the component to identify Splitter entities and their type.

- [ ] **2. Create `SplitterPlugin`:**
  - [ ] In `src/game/enemies/splitter.rs`, create a `SplitterPlugin`.
  - [ ] Add this plugin to the main `EnemiesPlugin` in `src/game/enemies/mod.rs`.

- [ ] **3. Implement `spawn_splitter` System:**
  - [ ] In `src/game/enemies/splitter.rs`, create a `spawn_splitter` system function (following the pattern of `spawn_ball`, `spawn_square`).
  - [ ] This system will spawn the initial "Queen" Splitter:
    - [ ] Use `generate_enemy_position` (from `src/utils.rs`) for placement; skip the spawn on `None`.
    - [ ] Add the `Enemy` marker component.
    - [ ] Add the `SplitterType::Queen` component.
    - [ ] Add appropriate physics components (`Collider`, `RigidBody`, etc.).
    - [ ] Add a `Mesh2dBundle` using an **Octagon** shape.
    - [ ] Add components for movement (see step 4).
  - [ ] Add `EnemyKind::Splitter` and a `SpawnRule` row (first level, weight) to `SPAWN_TABLE` (in `enemies/rng.rs`), and a `spawn_splitter` arm to the `match` in `spawn_enemies` (in `enemies/mod.rs`) ([032]).

- [ ] **4. Define Splitter Movement:**
  - [ ] Implement movement behavior where Splitters (both Queen and Soldiers) move in straight lines, changing direction randomly after a certain time or distance.
  - [ ] Implement the necessary components and systems (likely within `splitter.rs`) to handle this movement.
  - [ ] Soldiers should generally move faster than the Queen.

- [ ] **5. Create `SpawnSplitterSoldiers` Event:**
  - [ ] Define a new public event `SpawnSplitterSoldiers` in the main events module (`src/game/events.rs`).
  - [ ] This event does not need to carry any data. It simply signals that Soldiers should be spawned.

- [ ] **6. Modify `on_despawn_enemy` Observer:**
  - [ ] In `src/game/enemies/mod.rs`, update the `on_despawn_enemy` observer.
  - [ ] Query the entity being despawned for the `SplitterType` component.
  - [ ] **Before** despawning, check if the entity has the `SplitterType::Queen` component.
  - [ ] If it is a Queen, send the `SpawnSplitterSoldiers` event (defined in `src/game/events.rs`).
  - [ ] Continue with the existing logic: despawn the entity, increment `level_info.kill_count`, and send the `IncreasePoints` event.

- [ ] **7. Implement Soldier Spawning System:**
  - [ ] In `src/game/enemies/splitter.rs`, create a new system (e.g., `handle_spawn_splitter_soldiers`).
  - [ ] This system should read `SpawnSplitterSoldiers` events.
  - [ ] For each event received:
    - [ ] Spawn **two** new "Soldier" entities.
    - [ ] Use `generate_enemy_position` to find a random valid spawn location for each Soldier.
    - [ ] Each Soldier entity needs:
      - [ ] `Enemy` marker component.
      - [ ] `SplitterType::Soldier` component.
      - [ ] Appropriate physics components (`Collider`, `RigidBody`, etc.).
      - [ ] A `Mesh2dBundle` using an **Ellipse** shape.
      - [ ] Movement components (configured for higher speed than the Queen).

- [ ] **8. Refine `on_remove_enemy`:**
  - [ ] In `src/game/enemies/mod.rs`, **remove** the line `level_info.kill_count += 1;` from the `on_remove_enemy` observer to avoid double-counting kills. The count is now handled reliably in `on_despawn_enemy`.
  - [ ] Ensure the explosion effect created in `on_remove_enemy` works visually for both Queen (Octagon) and Soldier (Ellipse) shapes upon their removal.

---

**Future Improvements:**

- [ ] **Soldier Spawn Animation:** Implement a visual "bounce" or scaling effect when Soldiers spawn, similar to the effect used in `spawn_ball`.
- [ ] **Soldier Idle Animation:** Add a continuous animation to the Soldier's Ellipse shape, making it appear "wobbly" or "gooey" by subtly changing its size/proportions over time.
