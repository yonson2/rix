use bevy::{prelude::*, sprite::Anchor};

use super::{
    setup::LevelInfo,
    state::Phase,
    wall::{FIELD, WALL_THICKNESS, WALL_THICKNESS_TOP},
};

const MARGIN_TOP: f32 = WALL_THICKNESS_TOP * 1.35;
const MARGIN_SIDES: f32 = WALL_THICKNESS * 1.5;
const POINTS_MARGIN_TOP: f32 = 6.;
const POINTS_FONT_SIZE: FontSize = FontSize::Px(30.);
const SMALL_FONT_SIZE: FontSize = FontSize::Px(20.);
/// Above the claimed shapes, enemies and items.
const HUD_Z: f32 = 10.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Phase::Playing), setup_ui);
    app.add_systems(
        Update,
        update_ui.run_if(resource_changed::<LevelInfo>.and_then(in_state(Phase::Playing))),
    );
}

#[derive(Component)]
struct UiLevel;
#[derive(Component)]
struct UiPoints;
#[derive(Component)]
struct UiProgress;

/// The HUD is text in the world, just under the top wall, so it scales and letterboxes
/// with the field ([019]).
fn setup_ui(mut commands: Commands, level_info: Res<LevelInfo>, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/main.ttf");
    let top = FIELD.y / 2. - MARGIN_TOP;
    let side = FIELD.x / 2. - MARGIN_SIDES;
    let hud = |text: String, font_size, anchor, at: Vec2| {
        (
            DespawnOnExit(Phase::Playing),
            Text2d::new(text),
            TextFont {
                font: font.clone().into(),
                font_size,
                ..default()
            },
            TextColor::WHITE,
            anchor,
            Transform::from_translation(at.extend(HUD_Z)),
        )
    };

    commands.spawn((
        hud(
            format!("Level: {}", level_info.level),
            SMALL_FONT_SIZE,
            Anchor::TOP_LEFT,
            Vec2::new(-side, top),
        ),
        UiLevel,
    ));
    commands.spawn((
        hud(
            level_info.points.to_string(),
            POINTS_FONT_SIZE,
            Anchor::TOP_CENTER,
            Vec2::new(0., top - POINTS_MARGIN_TOP),
        ),
        UiPoints,
    ));
    commands.spawn((
        hud(
            format!(
                "{}/{}%",
                (level_info.current * 100.) as u8,
                (level_info.target * 100.) as u8
            ),
            SMALL_FONT_SIZE,
            Anchor::TOP_RIGHT,
            Vec2::new(side, top),
        ),
        UiProgress,
    ));
}

#[allow(clippy::type_complexity)]
fn update_ui(
    mut level_q: Query<&mut Text2d, (With<UiLevel>, Without<UiPoints>, Without<UiProgress>)>,
    mut points_q: Query<&mut Text2d, (With<UiPoints>, Without<UiLevel>, Without<UiProgress>)>,
    mut progress_q: Query<&mut Text2d, (With<UiProgress>, Without<UiPoints>, Without<UiLevel>)>,
    level_info: Res<LevelInfo>,
) {
    if let Ok(mut text) = level_q.single_mut() {
        **text = format!("Level: {}", level_info.level);
    }

    if let Ok(mut text) = points_q.single_mut() {
        **text = level_info.points.to_string();
    }

    if let Ok(mut text) = progress_q.single_mut() {
        **text = format!(
            "{}/{}%",
            (level_info.current * 100.) as u8,
            (level_info.target * 100.) as u8
        );
    }
}
