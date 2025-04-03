use avian2d::prelude::*;
use basic_bomb::{BasicBombPlugin, spawn_basic_bomb};
use bevy::{ecs::system::SystemId, prelude::*, window::PrimaryWindow};
use bevy_light_2d::plugin::Light2dPlugin;
use std::fmt::Debug;

mod basic_bomb;

use super::{
    events::{ActivateBasicBomb, ActivateItem},
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
            (handle_collision, activate_item, despawn_item).run_if(in_state(IsPaused::Running)),
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

fn on_pressed_item(
    click: Trigger<Pointer<Down>>,
    items: Query<&Item>,
    mut events: EventWriter<ActivateItem>,
) {
    if let Ok(item) = items.get(click.target) {
        events.send(ActivateItem(click.target, *item));
    }
}
