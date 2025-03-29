use super::{polygon::PolygonData, wall::WallSide};
use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};

#[derive(Event, Debug, Clone, Deref, DerefMut)]
pub struct DrawLine(pub Vec<Vec2>);

#[derive(Event, Debug, Clone, Deref, DerefMut)]
pub struct BuildPolygon(pub PolygonData);

#[derive(Event, Debug, Clone, Deref, DerefMut)]
pub struct DrawPolygon {
    #[deref]
    pub path: Vec<Vec2>,
    pub meta: ShapeMeta,
}

#[derive(Debug, Clone, Copy, Component)]
#[component(on_add = on_add_shape_meta)]
pub struct ShapeMeta {
    pub start_wall: WallSide,
    pub start_point: Vec2,
}

#[derive(Event, Debug, Clone)]
pub struct ClearLines;

#[derive(Event, Debug, Clone)]
pub struct Die;

#[derive(Event, Debug, Clone, Deref)]
pub struct DespawnEnemy(pub Entity);

fn on_add_shape_meta(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    info!("YAY you added");
}
