use super::items::Item;
use bevy::prelude::*;

#[derive(Message, Debug, Clone, Deref, DerefMut)]
pub struct DrawLine(pub Vec<Vec2>);

/// A finished line, from the free boundary to the free boundary.
#[derive(Message, Debug, Clone, Deref, DerefMut)]
pub struct BuildPolygon(pub Vec<Vec2>);

/// A claimed piece, counter-clockwise.
#[derive(Message, Debug, Clone, Deref, DerefMut)]
pub struct DrawPolygon(pub Vec<Vec2>);

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
