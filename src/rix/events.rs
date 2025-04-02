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
}

#[derive(Event, Debug, Clone)]
pub struct ClearLines;

#[derive(Event, Debug, Clone)]
pub struct Die;

#[derive(Event, Debug, Clone, Deref)]
pub struct DespawnEnemy(pub Entity);

#[derive(Event, Debug, Clone, Deref, DerefMut, Default)]
pub struct IncreasePoints(pub u128);

#[derive(Event, Debug, Clone, Deref, DerefMut, Default)]
pub struct UpdatePoints(pub PointsProgress);

#[derive(Event, Debug, Clone, Deref, DerefMut)]
pub struct InputPosition(pub Vec2);

#[derive(Debug, Clone, Copy, Default)]
pub struct PointsProgress {
    pub from: f32,
    pub to: f32,
}

impl UpdatePoints {
    pub fn new(from: f32, to: f32) -> Self {
        UpdatePoints(PointsProgress { from, to })
    }
}

pub struct EventsPlugin;
impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DrawLine>()
            .add_event::<BuildPolygon>()
            .add_event::<ClearLines>()
            .add_event::<DrawPolygon>()
            .add_event::<Die>()
            .add_event::<DespawnEnemy>()
            .add_event::<UpdatePoints>()
            .add_event::<InputPosition>()
            .add_event::<IncreasePoints>();
    }
}
