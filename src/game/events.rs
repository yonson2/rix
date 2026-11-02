use super::{items::Item, polygon::PolygonData, wall::WallSide};
use bevy::prelude::*;

#[derive(Message, Debug, Clone, Deref, DerefMut)]
pub struct DrawLine(pub Vec<Vec2>);

#[derive(Message, Debug, Clone, Deref, DerefMut)]
pub struct BuildPolygon(pub PolygonData);

#[derive(Message, Debug, Clone, Deref, DerefMut)]
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

#[derive(Message, Debug, Clone)]
pub struct ClearLines;

#[derive(Message, Debug, Clone)]
pub struct Die;

#[derive(Event, Debug, Clone, Deref)]
pub struct DespawnEnemy(pub Entity);

#[derive(Message, Debug, Clone)]
pub struct ActivateItem(pub Entity, pub Item);

#[derive(Message, Debug, Clone)]
pub struct ActivateBasicBomb;

#[derive(Message, Debug, Clone, Deref, DerefMut, Default)]
pub struct IncreasePoints(pub u128);

#[derive(Message, Debug, Clone, Deref, DerefMut, Default)]
pub struct UpdatePoints(pub PointsProgress);

#[derive(Message, Debug, Clone, Deref, DerefMut)]
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

pub(super) fn plugin(app: &mut App) {
    app.add_message::<DrawLine>()
        .add_message::<BuildPolygon>()
        .add_message::<ClearLines>()
        .add_message::<DrawPolygon>()
        .add_message::<Die>()
        .add_message::<UpdatePoints>()
        .add_message::<InputPosition>()
        .add_message::<IncreasePoints>()
        .add_message::<ActivateItem>()
        .add_message::<ActivateBasicBomb>();
}
