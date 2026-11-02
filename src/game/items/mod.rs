use avian2d::prelude::*;
use basic_bomb::spawn_basic_bomb;
use bevy::prelude::*;
use std::fmt::Debug;

mod basic_bomb;

use crate::theme::{FontAssets, palette::NEON_WHITE};

use super::{
    AppSystems, PausableSystems,
    effects::GlowMaterial,
    events::{ActivateBasicBomb, ActivateItem, IncreasePoints},
    level::DespawnAfter,
    polygon::TempLineSegment,
    setup::GameRng,
    state::Phase,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(basic_bomb::plugin);
    // After the enemies: both draw from the level RNG, so a seed needs one order.
    app.add_systems(
        OnEnter(Phase::Playing),
        spawn_items.after(super::enemies::spawn_enemies),
    );
    // TODO: When should we spawn items?
    // app.add_systems(OnEnter(Phase::Playing), spawn_basic_bomb);
    app.add_systems(
        FixedUpdate,
        (handle_collision, activate_item, despawn_item)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        (blink_items, display_countdown)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

//TODO: Right now this only spawns when entering a level.
//This is bad, it should be at any point in time while playing.
fn spawn_items(world: &mut World) {
    if world.resource_mut::<GameRng>().bool()
        && let Err(e) = world.run_system_cached(spawn_basic_bomb)
    {
        error!("Error running system: {e}");
    }
}

fn activate_item(
    mut commands: Commands,
    mut activate: MessageReader<ActivateItem>,
    mut basic_bomb: MessageWriter<ActivateBasicBomb>,
    mut points: MessageWriter<IncreasePoints>,
) {
    for e in activate.read() {
        info!("Activating item {}", e.0);
        if let Ok(mut ec) = commands.get_entity(e.0) {
            ec.despawn();
            match e.1 {
                Item::BasicBomb => basic_bomb.write(ActivateBasicBomb),
            };
        }
        points.write(IncreasePoints(super::level::ITEM_POINTS));
    }
}

// System to make items blink with increasing frequency as they approach despawn time
fn blink_items(
    time: Res<Time>,
    items: Query<(&DespawnAfter, &MeshMaterial2d<GlowMaterial>), With<Item>>,
    mut materials: ResMut<Assets<GlowMaterial>>,
) {
    const BLINK_START_TIME: f32 = 3.0;

    const MIN_FREQUENCY: f32 = 0.4;
    const MAX_FREQUENCY: f32 = 2.0;

    for (despawn_after, mesh_material) in &items {
        if let Some(mut material) = materials.get_mut(&mesh_material.0) {
            let total_duration = despawn_after.duration().as_secs_f32();
            let remaining_time = total_duration - despawn_after.elapsed_secs();

            let max_intensity = 2.5;

            if remaining_time <= BLINK_START_TIME {
                let blink_progress = remaining_time / BLINK_START_TIME;

                let time_factor: f32 = 1.0 - blink_progress;

                let frequency =
                    MIN_FREQUENCY + time_factor.powf(1.2) * (MAX_FREQUENCY - MIN_FREQUENCY);
                let phase = time.elapsed_secs() * frequency;

                let square_wave = if (phase * std::f32::consts::TAU).sin() > 0.0 {
                    1.0
                } else {
                    0.0
                };

                material.intensity = square_wave * max_intensity;
            } else {
                material.intensity = max_intensity;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn display_countdown(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    items: Query<(Entity, &DespawnAfter), With<Item>>,
    mut existing_texts: Query<
        (
            Entity,
            &mut Text2d,
            &mut TextFont,
            &mut Transform,
            &mut CountdownText,
            &ChildOf,
        ),
        Without<Item>,
    >,
) {
    const BASE_FONT_SIZE: f32 = 40.0;
    const MAX_FONT_SIZE: f32 = BASE_FONT_SIZE * 1.5;

    let font = &fonts.main;

    let mut parents_with_text = Vec::new();

    for (text_entity, mut text, mut text_font, mut transform, mut countdown, parent) in
        &mut existing_texts
    {
        let parent_entity = parent.parent();
        parents_with_text.push(parent_entity);

        if let Ok((_, despawn_after)) = items.get(parent_entity) {
            let remaining_seconds = despawn_after.remaining_secs().ceil() as i32;

            if remaining_seconds <= 5 && remaining_seconds > 0 {
                if countdown.last_value != remaining_seconds {
                    countdown.last_value = remaining_seconds;
                    text.0 = remaining_seconds.to_string();
                }

                let fractional_time =
                    despawn_after.remaining_secs() - remaining_seconds as f32 + 1.0;
                let second_progress = f32::clamp(1.0 - fractional_time, 0.0, 1.0);

                let size_factor = 1.0 + (1.0 - second_progress) * (1.0 - second_progress) * 1.5;

                let dynamic_size = BASE_FONT_SIZE * size_factor;
                text_font.font_size = FontSize::Px(dynamic_size);

                transform.translation.z = 0.1 + (1.0 - second_progress) * 0.05;
            } else if commands.get_entity(text_entity).is_ok() {
                commands.entity(text_entity).despawn();
            }
        } else if commands.get_entity(text_entity).is_ok() {
            commands.entity(text_entity).despawn();
        }
    }

    for (entity, despawn_after) in items.iter() {
        if parents_with_text.contains(&entity) {
            continue;
        }

        let remaining_seconds = despawn_after.remaining_secs().ceil() as i32;

        if remaining_seconds <= 5 && remaining_seconds > 0 {
            let text_entity = commands
                .spawn((
                    DespawnOnExit(Phase::Playing),
                    Text2d::new(remaining_seconds.to_string()),
                    TextFont {
                        font: font.clone().into(),
                        font_size: FontSize::Px(MAX_FONT_SIZE),
                        ..default()
                    },
                    TextColor(NEON_WHITE),
                    Transform::from_xyz(0.0, 0.0, 0.15),
                    CountdownText {
                        last_value: remaining_seconds,
                    },
                ))
                .id();

            if commands.get_entity(entity).is_ok() {
                commands.entity(entity).add_child(text_entity);
            } else {
                commands.entity(text_entity).despawn();
            }
        }
    }
}

#[derive(Component)]
struct CountdownText {
    last_value: i32,
}

fn despawn_item(mut commands: Commands, item_query: Query<(Entity, &Transform), With<Item>>) {
    let half_height = super::wall::FIELD.y / 2.;
    for (e, t) in item_query.iter() {
        if t.translation.truncate().y < -half_height - 100. {
            info!("Bye bye my little friend {e}");
            if let Ok(mut ec) = commands.get_entity(e) {
                ec.despawn();
            }
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub enum Item {
    BasicBomb,
}

fn handle_collision(
    mut collision_events: MessageReader<CollisionStart>,
    mut activate: MessageWriter<ActivateItem>,
    items: Query<&Item>,
    item_query: Query<Entity, With<Item>>,
    line_query: Query<Entity, With<TempLineSegment>>,
) {
    for CollisionStart {
        collider1: entity1,
        collider2: entity2,
        ..
    } in collision_events.read()
    {
        let entity1 = *entity1;
        let entity2 = *entity2;
        let is_item = item_query.contains(entity1) || item_query.contains(entity2);
        let is_player = line_query.contains(entity1) || line_query.contains(entity2);

        if is_player && is_item {
            let item = if item_query.contains(entity1) {
                entity1
            } else {
                entity2
            };

            if let Ok(item_type) = items.get(item) {
                activate.write(ActivateItem(item, *item_type));
            } else {
                error!("I should have gotten an Item");
            }
        }
    }
}
