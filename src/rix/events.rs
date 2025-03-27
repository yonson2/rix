use super::polygon::PolygonData;
use bevy::prelude::*;

#[derive(Event, Debug, Clone, Deref, DerefMut)]
pub struct DrawLine(pub Vec<Vec2>);

#[derive(Event, Debug, Clone, Deref, DerefMut)]
pub struct BuildPolygon(pub PolygonData);

#[derive(Event, Debug, Clone, Deref, DerefMut)]
pub struct DrawPolygon(pub Vec<Vec2>);

#[derive(Event, Debug, Clone)]
pub struct ClearLines;

#[derive(Event, Debug, Clone)]
pub struct Die;

#[derive(Event, Debug, Clone, Deref)]
pub struct DespawnEnemy(pub Entity);
