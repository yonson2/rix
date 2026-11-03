use avian2d::prelude::*;
use basic_bomb::spawn_basic_bomb;
use bevy::prelude::*;
use std::fmt::Debug;

pub mod auto_line;
mod basic_bomb;
pub mod multiplier;
pub mod time_stop;

use crate::{
    shape::{self, Shape},
    theme::{FontAssets, palette::NEON_WHITE},
    utils::generate_enemy_position,
};

use super::{
    AppSystems, PausableSystems,
    effects::GlowMaterial,
    events::{ActivateAutoLine, ActivateBasicBomb, ActivateItem, IncreasePoints},
    get_item_layer,
    level::DespawnAfter,
    polygon::TempLineSegment,
    setup::{GameRng, Run},
    state::Phase,
    wall::Field,
};
use multiplier::ScoreBoost;
use time_stop::TimeStop;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((basic_bomb::plugin, multiplier::plugin, time_stop::plugin));
    // After the enemies: both draw from the level RNG, so a seed needs one order.
    app.init_resource::<ItemTimer>();
    app.add_systems(
        OnEnter(Phase::Playing),
        (
            spawn_items.after(super::enemies::spawn_enemies),
            reset_item_timer,
            spawn_effects_label,
        ),
    );
    app.add_systems(
        FixedUpdate,
        roll_item
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems),
    );
    app.add_systems(
        FixedUpdate,
        (handle_collision, activate_item, despawn_item)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        (blink_items, display_countdown, show_effects)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// Seconds of play between two item rolls ([033]); [047] tunes it.
const ITEM_INTERVAL: f32 = 10.;
/// The chance that a roll spawns an item.
const ITEM_CHANCE: f32 = 0.5;

/// Times the item rolls of a level.
#[derive(Resource, Deref, DerefMut)]
struct ItemTimer(Timer);

impl Default for ItemTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(ITEM_INTERVAL, TimerMode::Repeating))
    }
}

fn reset_item_timer(mut timer: ResMut<ItemTimer>) {
    timer.reset();
}

/// Each item and the first level it spawns on ([046]). New kinds start after level 1, so
/// level 1 keeps its draws and its replays.
const ITEM_LEVELS: &[(Item, usize)] = &[
    (Item::BasicBomb, 1),
    (Item::Multiplier, 2),
    (Item::TimeStop, 3),
    (Item::AutoLine, 4),
];
/// Shape items are a dark token with an icon ([011] Q5), as big as a bomb's collider and
/// there as long.
const TOKEN_SIZE: f32 = super::enemies::square::SQUARE_SIZE;
const TOKEN_SECS: f32 = 7.;
/// The effects label sits under the HUD bar.
const EFFECTS_Y: f32 = 84.;

/// An even pick of the items of `level`; with one item it draws nothing. It draws `u64`
/// so a seed picks the same on wasm, as `pick_kind` does.
fn pick_item(level: usize, rng: &mut fastrand::Rng) -> Item {
    let items: Vec<Item> = ITEM_LEVELS
        .iter()
        .filter(|(_, first)| level >= *first)
        .map(|(item, _)| *item)
        .collect();
    if items.len() == 1 {
        return items[0];
    }
    items[rng.u64(0..items.len() as u64) as usize]
}

fn spawn_item(commands: &mut Commands, item: Item) {
    let icon = match item {
        Item::BasicBomb => {
            commands.run_system_cached(spawn_basic_bomb);
            return;
        }
        Item::Multiplier => multiplier::icon(),
        Item::TimeStop => time_stop::icon(),
        Item::AutoLine => auto_line::icon(),
    };
    commands.run_system_cached_with(spawn_token, (item, icon));
}

fn spawn_token(
    In((item, icon)): In<(Item, Shape)>,
    mut commands: Commands,
    mut rng: ResMut<GameRng>,
    spatial_query: SpatialQuery,
    field: Res<Field>,
) {
    let collider = Collider::rectangle(TOKEN_SIZE, TOKEN_SIZE);
    let Some(pos) = generate_enemy_position(&spatial_query, &mut rng, &collider, &field) else {
        return;
    };
    commands.spawn((
        DespawnOnExit(Phase::Playing),
        item,
        Shape::new(shape::rounded_rectangle(Vec2::splat(TOKEN_SIZE), 8.))
            .fill(Color::BLACK.with_alpha(0.6))
            .stroke(NEON_WHITE, 2.),
        collider,
        RigidBody::Dynamic,
        GravityScale(0.),
        LinearDamping(0.33),
        Transform::from_xyz(pos.x, pos.y, 1.),
        DespawnAfter::from_seconds(TOKEN_SECS),
        get_item_layer(),
        children![(icon, Transform::from_xyz(0., 0., 0.05))],
    ));
}

/// Every `ITEM_INTERVAL` of play, an item spawns with `ITEM_CHANCE`.
fn roll_item(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ItemTimer>,
    mut rng: ResMut<GameRng>,
    run: Res<Run>,
) {
    if timer.tick(time.delta()).just_finished() && rng.f32() < ITEM_CHANCE {
        let item = pick_item(run.level, &mut rng);
        spawn_item(&mut commands, item);
    }
}

/// A level starts with an item on a coin flip.
fn spawn_items(world: &mut World) {
    if world.resource_mut::<GameRng>().bool() {
        let level = world.resource::<Run>().level;
        let item = pick_item(level, &mut world.resource_mut::<GameRng>());
        spawn_item(&mut world.commands(), item);
        world.flush();
    }
}

#[derive(Component)]
struct EffectsLabel;

fn spawn_effects_label(mut commands: Commands, fonts: Res<FontAssets>, field: Res<Field>) {
    commands.spawn((
        EffectsLabel,
        DespawnOnExit(Phase::Playing),
        Text2d::default(),
        TextFont {
            font: fonts.main.clone().into(),
            font_size: FontSize::Px(24.),
            ..default()
        },
        TextColor(NEON_WHITE),
        Transform::from_xyz(0., field.y / 2. - EFFECTS_Y, 10.),
    ));
}

/// Draws only: the timed effects and their seconds left.
fn show_effects(
    boost: Option<Res<ScoreBoost>>,
    stop: Option<Res<TimeStop>>,
    mut labels: Query<&mut Text2d, With<EffectsLabel>>,
) {
    let text = effects_text(
        boost.map(|b| b.remaining_secs()),
        stop.map(|s| s.remaining_secs()),
    );
    for mut label in &mut labels {
        if label.0 != text {
            label.0 = text.clone();
        }
    }
}

fn effects_text(boost: Option<f32>, stop: Option<f32>) -> String {
    let boost = boost.map(|s| format!("x{} {}", multiplier::BOOST_FACTOR, s.ceil()));
    let stop = stop.map(|s| format!("STOP {}", s.ceil()));
    [boost, stop]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("   ")
}

fn activate_item(
    mut commands: Commands,
    mut activate: MessageReader<ActivateItem>,
    mut basic_bomb: MessageWriter<ActivateBasicBomb>,
    mut auto_line: MessageWriter<ActivateAutoLine>,
    mut points: MessageWriter<IncreasePoints>,
) {
    // Several line segments can touch one item in a tick; the despawn waits ([083]).
    let mut fired = vec![];
    for e in activate.read() {
        if fired.contains(&e.0) {
            continue;
        }
        fired.push(e.0);
        if let Ok(mut ec) = commands.get_entity(e.0) {
            info!("Activating item {}", e.0);
            ec.despawn();
            match e.1 {
                Item::BasicBomb => {
                    basic_bomb.write(ActivateBasicBomb);
                }
                Item::Multiplier => commands.insert_resource(ScoreBoost::default()),
                Item::TimeStop => commands.insert_resource(TimeStop::default()),
                Item::AutoLine => {
                    auto_line.write(ActivateAutoLine);
                }
            }
            points.write(IncreasePoints(super::level::ITEM_POINTS));
        }
    }
}

// System to make items blink with increasing frequency as they approach despawn time
fn blink_items(
    time: Res<Time>,
    items: Query<(&DespawnAfter, &MeshMaterial2d<GlowMaterial>), With<Item>>,
    mut tokens: Query<(&DespawnAfter, &mut Visibility), (With<Item>, With<Shape>)>,
    mut materials: ResMut<Assets<GlowMaterial>>,
) {
    let now = time.elapsed_secs();
    for (despawn_after, mesh_material) in &items {
        if let Some(mut material) = materials.get_mut(&mesh_material.0) {
            let max_intensity = 2.5;
            material.intensity = if blink_on(despawn_after.remaining_secs(), now) {
                max_intensity
            } else {
                0.
            };
        }
    }
    for (despawn_after, mut visibility) in &mut tokens {
        *visibility = if blink_on(despawn_after.remaining_secs(), now) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Whether an item with `remaining` seconds shows at time `now`: in its last seconds it
/// blinks faster and faster.
fn blink_on(remaining: f32, now: f32) -> bool {
    const BLINK_START_TIME: f32 = 3.0;

    const MIN_FREQUENCY: f32 = 0.4;
    const MAX_FREQUENCY: f32 = 2.0;

    if remaining > BLINK_START_TIME {
        return true;
    }
    let time_factor: f32 = 1.0 - remaining / BLINK_START_TIME;
    let frequency = MIN_FREQUENCY + ops::powf(time_factor, 1.2) * (MAX_FREQUENCY - MIN_FREQUENCY);
    ops::sin(now * frequency * std::f32::consts::TAU) > 0.0
}

fn display_countdown(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    // A token blinks instead: a number would hide its icon.
    items: Query<(Entity, &DespawnAfter), (With<Item>, Without<Shape>)>,
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

fn despawn_item(
    mut commands: Commands,
    item_query: Query<(Entity, &Transform), With<Item>>,
    field: Res<super::wall::Field>,
) {
    let half_height = field.y / 2.;
    for (e, t) in item_query.iter() {
        if t.translation.truncate().y < -half_height - 100. {
            info!("Bye bye my little friend {e}");
            if let Ok(mut ec) = commands.get_entity(e) {
                ec.despawn();
            }
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    BasicBomb,
    Multiplier,
    TimeStop,
    AutoLine,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn a_bomb_spawns_during_play_when_a_roll_succeeds() {
        let seed = (0..)
            .find(|&s| fastrand::Rng::with_seed(s).f32() < ITEM_CHANCE)
            .unwrap();
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::mesh::MeshPlugin,
            TransformPlugin,
            PhysicsPlugins::default(),
        ))
        .init_asset::<Image>()
        .init_asset::<GlowMaterial>()
        .init_resource::<basic_bomb::ItemAssets>()
        .init_resource::<ItemTimer>()
        .init_resource::<crate::game::wall::Field>()
        .insert_resource(GameRng::new(seed))
        .insert_resource(Run::with_seed(seed))
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(0.25),
        ))
        .add_systems(Update, roll_item);
        app.finish();
        let items = |app: &mut App, updates: u32| {
            for _ in 0..updates {
                app.update();
            }
            let mut query = app.world_mut().query::<&Item>();
            query.iter(app.world()).count()
        };

        assert_eq!(items(&mut app, 38), 0, "no roll before the interval");
        assert_eq!(items(&mut app, 4), 1, "the first roll spawns a bomb");
    }

    #[test]
    fn level_1_picks_the_bomb_without_a_draw_and_level_4_picks_all_four() {
        let mut rng = fastrand::Rng::with_seed(7);
        assert_eq!(pick_item(1, &mut rng), Item::BasicBomb);
        assert_eq!(rng.u64(..), fastrand::Rng::with_seed(7).u64(..), "no draw");

        let mut rng = fastrand::Rng::with_seed(7);
        let mut picks: Vec<Item> = (0..40).map(|_| pick_item(4, &mut rng)).collect();
        picks.sort_by_key(|item| *item as u8);
        picks.dedup();
        assert_eq!(
            picks,
            [
                Item::BasicBomb,
                Item::Multiplier,
                Item::TimeStop,
                Item::AutoLine
            ]
        );
        let mut rng = fastrand::Rng::with_seed(7);
        assert!((0..40).all(|_| pick_item(2, &mut rng) != Item::TimeStop));
    }

    #[test]
    fn a_gem_and_an_hourglass_start_their_effects_and_an_arrow_its_shot() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.init_resource::<Messages<ActivateItem>>();
        world.init_resource::<Messages<ActivateBasicBomb>>();
        world.init_resource::<Messages<ActivateAutoLine>>();
        world.init_resource::<Messages<IncreasePoints>>();
        for item in [Item::Multiplier, Item::TimeStop, Item::AutoLine] {
            let entity = world.spawn(item).id();
            world.write_message(ActivateItem(entity, item));
        }
        world.run_system_once(activate_item).unwrap();
        assert!(world.contains_resource::<ScoreBoost>());
        assert!(world.contains_resource::<TimeStop>());
        assert_eq!(world.resource::<Messages<ActivateAutoLine>>().len(), 1);
        assert_eq!(world.resource::<Messages<IncreasePoints>>().len(), 3);
    }

    #[test]
    fn the_label_shows_each_effect_with_its_seconds_left() {
        assert_eq!(effects_text(None, None), "");
        assert_eq!(effects_text(Some(9.2), None), "x2 10");
        assert_eq!(effects_text(Some(0.5), Some(3.)), "x2 1   STOP 3");
    }

    #[test]
    fn an_item_touched_by_many_segments_in_one_tick_fires_once() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.init_resource::<Messages<ActivateItem>>();
        world.init_resource::<Messages<ActivateBasicBomb>>();
        world.init_resource::<Messages<ActivateAutoLine>>();
        world.init_resource::<Messages<IncreasePoints>>();
        let bomb = world.spawn(Item::BasicBomb).id();
        for _ in 0..7 {
            world.write_message(ActivateItem(bomb, Item::BasicBomb));
        }
        world.run_system_once(activate_item).unwrap();
        assert_eq!(world.resource::<Messages<ActivateBasicBomb>>().len(), 1);
        assert_eq!(world.resource::<Messages<IncreasePoints>>().len(), 1);
        assert!(world.get_entity(bomb).is_err());

        world.write_message(ActivateItem(bomb, Item::BasicBomb));
        world.run_system_once(activate_item).unwrap();
        assert_eq!(
            world.resource::<Messages<IncreasePoints>>().len(),
            1,
            "a gone item pays nothing"
        );
    }
}
