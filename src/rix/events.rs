use super::{polygon::PolygonData, wall::WallSide};
use bevy::prelude::*;

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
pub struct ShapeMeta {
    pub start_wall: WallSide,
    pub start_point: Vec2,
    pub from_poly: Option<Entity>,
}

#[derive(Event, Debug, Clone)]
pub struct ClearLines;

#[derive(Event, Debug, Clone)]
pub struct Die;

#[derive(Event, Debug, Clone, Deref)]
pub struct DespawnEnemy(pub Entity);
