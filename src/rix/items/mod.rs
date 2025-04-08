use avian2d::prelude::*;
use basic_bomb::{BasicBombPlugin, spawn_basic_bomb};
use bevy::{ecs::system::SystemId, prelude::*, window::PrimaryWindow};
use bevy_light_2d::plugin::Light2dPlugin;
use std::fmt::Debug;

mod basic_bomb;

use crate::colors::NEON_WHITE;

use super::{
    RixComponent,
    effects::GlowMaterial,
    events::{ActivateBasicBomb, ActivateItem},
    level::DespawnAfter,
    polygon::TempLineSegment,
    setup::LevelInfo,
    state::{AppState, IsPaused},
};

pub struct ItemsPlugin;
impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Light2dPlugin);
        app.add_plugins(BasicBombPlugin);
        app.add_systems(OnEnter(AppState::InGame), spawn_items);
        // TODO: When should we spawn items?
        // app.add_systems(OnEnter(AppState::InGame), spawn_basic_bomb);
        app.add_systems(
            Update,
            (
                handle_collision,
                activate_item,
                despawn_item,
                blink_items,
                display_countdown,
            )
                .run_if(in_state(IsPaused::Running)),
        );
    }
}

#[derive(Resource, Clone, Copy)]
pub struct ItemSystems {
    pub spawn_basic_bomb: SystemId,
}

pub fn register_item_systems(world: &mut World) {
    let spawn_basic_bomb = world.register_system_cached(spawn_basic_bomb);
    let systems = ItemSystems { spawn_basic_bomb };
    world.insert_resource(systems);
}

//TODO: Right now this only spawns when entering a level.
//This is bad, it should be at any point in time while playing.
fn spawn_items(world: &mut World) {
    let systems = {
        let item_systems = world.resource::<ItemSystems>();
        [item_systems.spawn_basic_bomb]
    };

    let should_spawn = {
        let mut level_info = world.resource_mut::<LevelInfo>();
        level_info.rng.bool()
    };

    if should_spawn {
        for system_id in systems {
            if let Err(e) = world.run_system(system_id) {
                error!("Error running system: {e}");
            }
        }
    }
}

fn activate_item(
    mut commands: Commands,
    mut activate: EventReader<ActivateItem>,
    mut basic_bomb: EventWriter<ActivateBasicBomb>,
) {
    for e in activate.read() {
        info!("Activating item {}", e.0);
        if let Some(ec) = commands.get_entity(e.0) {
            ec.despawn_recursive();
            match e.1 {
                Item::BasicBomb => basic_bomb.send(ActivateBasicBomb),
            };
        }
    }
}

// System to make items blink with increasing frequency as they approach despawn time
fn blink_items(
    time: Res<Time>,
    items: Query<(&DespawnAfter, &MeshMaterial2d<GlowMaterial>), With<Item>>,
    mut materials: ResMut<Assets<GlowMaterial>>,
) {
    // Only start blinking when there are 3 seconds or less remaining
    const BLINK_START_TIME: f32 = 3.0;

    // Frequency range (blinks per second)
    const MIN_FREQUENCY: f32 = 0.4; // Starting frequency (slower initial blink)
    const MAX_FREQUENCY: f32 = 2.0; // Maximum frequency (much slower)

    for (despawn_after, mesh_material) in &items {
        if let Some(material) = materials.get_mut(&mesh_material.0) {
            // Calculate how much time is left before despawn
            let total_duration = despawn_after.duration().as_secs_f32();
            let remaining_time = total_duration - despawn_after.elapsed_secs();

            // Default intensity when not blinking
            let max_intensity = 2.5;

            // Only start blinking when remaining time is less than BLINK_START_TIME
            if remaining_time <= BLINK_START_TIME {
                // Calculate progress within the blinking period (0.0 -> 1.0)
                let blink_progress = remaining_time / BLINK_START_TIME;

                // Reverse progress for frequency calculation (0.0 when starting to blink, 1.0 at despawn)
                let time_factor: f32 = 1.0 - blink_progress;

                // Subtle, natural progression from MIN to MAX frequency
                // Using a small power for a gentle acceleration curve
                let frequency =
                    MIN_FREQUENCY + time_factor.powf(1.2) * (MAX_FREQUENCY - MIN_FREQUENCY);

                // Calculate the phase based on elapsed time and frequency
                let phase = time.elapsed_secs() * frequency;

                // Use a square wave for sharper on/off transitions
                let square_wave = if (phase * std::f32::consts::TAU).sin() > 0.0 {
                    1.0
                } else {
                    0.0
                };

                // Full intensity or zero based on square wave
                material.intensity = square_wave * max_intensity;
            } else {
                // Before blinking starts, maintain constant intensity
                material.intensity = max_intensity;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
// System to display countdown number on items
fn display_countdown(
    mut commands: Commands, // Make commands mutable
    asset_server: Res<AssetServer>,
    items: Query<(Entity, &DespawnAfter), With<Item>>,
    mut existing_texts: Query<
        (
            Entity,
            &mut Text2d,
            &mut TextFont,
            &mut Transform,
            &mut CountdownText,
            &Parent,
        ),
        Without<Item>,
    >,
) {
    // Font size constants
    const BASE_FONT_SIZE: f32 = 40.0;
    const MAX_FONT_SIZE: f32 = BASE_FONT_SIZE * 1.5;

    let font = asset_server.load("fonts/main.ttf");

    // Track which parent items already have countdown text
    let mut parents_with_text = Vec::new();

    // Update existing countdown texts
    for (text_entity, mut text, mut text_font, mut transform, mut countdown, parent) in
        &mut existing_texts
    {
        let parent_entity = parent.get();
        parents_with_text.push(parent_entity);

        if let Ok((_, despawn_after)) = items.get(parent_entity) {
            let remaining_seconds = despawn_after.remaining_secs().ceil() as i32;

            // Only show text for the first digit of remaining seconds (5, 4, 3, 2, 1)
            if remaining_seconds <= 5 && remaining_seconds > 0 {
                // Update the text if the countdown has changed
                if countdown.last_value != remaining_seconds {
                    countdown.last_value = remaining_seconds;
                    text.0 = remaining_seconds.to_string();
                }

                // Calculate progress within the current second (0.0 to 1.0)
                let fractional_time =
                    despawn_after.remaining_secs() - remaining_seconds as f32 + 1.0;
                let second_progress = f32::clamp(1.0 - fractional_time, 0.0, 1.0);

                // Inverse of progress for font size (starts large, shrinks down)
                // Using quadratic easing for dramatic effect at the start
                let size_factor = 1.0 + (1.0 - second_progress) * (1.0 - second_progress) * 1.5;

                // Calculate font size (starts big, ends at base size)
                let dynamic_size = BASE_FONT_SIZE * size_factor;
                text_font.font_size = dynamic_size;

                // Adjust z position to ensure larger text stays visible
                transform.translation.z = 0.1 + (1.0 - second_progress) * 0.05;
            } else {
                // Text is no longer needed, safely despawn it
                if commands.get_entity(text_entity).is_some() {
                    commands.entity(text_entity).despawn_recursive();
                }
            }
        } else {
            // Parent item no longer exists, safely despawn the text
            if commands.get_entity(text_entity).is_some() {
                commands.entity(text_entity).despawn_recursive();
            }
        }
    }

    // Spawn new countdown texts only for items that don't already have text
    for (entity, despawn_after) in items.iter() {
        // Skip if this item already has countdown text
        if parents_with_text.contains(&entity) {
            continue;
        }

        // Only spawn text if within the 5-second countdown period
        let remaining_seconds = despawn_after.remaining_secs().ceil() as i32;

        if remaining_seconds <= 5 && remaining_seconds > 0 {
            // Initial font size is at maximum
            let text_entity = commands
                .spawn((
                    RixComponent,
                    Text2d::new(remaining_seconds.to_string()),
                    TextFont {
                        font: font.clone(),
                        font_size: MAX_FONT_SIZE, // Start with maximum size
                        ..default()
                    },
                    TextColor(NEON_WHITE), // Start with low opacity
                    // Position text in the center of the item with a small z-offset
                    Transform::from_xyz(0.0, 0.0, 0.15),
                    CountdownText {
                        last_value: remaining_seconds,
                    },
                ))
                .id();

            // Add the text entity as a child of the item, safely
            if commands.get_entity(entity).is_some() {
                commands.entity(entity).add_child(text_entity);
            } else {
                // If parent entity no longer exists, despawn the text we just created
                commands.entity(text_entity).despawn_recursive();
            }
        }
    }
}

// Component to track countdown value to avoid unnecessary text updates
#[derive(Component)]
struct CountdownText {
    last_value: i32,
}

fn despawn_item(
    mut commands: Commands,
    item_query: Query<(Entity, &Transform), With<Item>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let half_height = window.height() / 2.;
    for (e, t) in item_query.iter() {
        if t.translation.truncate().y < -half_height - 100. {
            info!("Bye bye my little friend {e}");
            if let Some(ec) = commands.get_entity(e) {
                ec.despawn_recursive();
            }
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum Item {
    BasicBomb,
}

fn handle_collision(
    mut collision_events: EventReader<Collision>,
    mut activate: EventWriter<ActivateItem>,
    items: Query<&Item>,
    item_query: Query<Entity, With<Item>>,
    line_query: Query<Entity, With<TempLineSegment>>,
) {
    for Collision(contacts) in collision_events.read() {
        let entity1 = contacts.entity1;
        let entity2 = contacts.entity2;
        let is_item = item_query.contains(entity1) || item_query.contains(entity2);
        let is_player = line_query.contains(entity1) || line_query.contains(entity2);

        if is_player && is_item {
            let item = if item_query.contains(entity1) {
                entity1
            } else {
                entity2
            };

            if let Ok(item_type) = items.get(item) {
                activate.send(ActivateItem(item, *item_type));
            } else {
                error!("I should have gotten an Item");
            }
        }
    }
}
