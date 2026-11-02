use bevy::{prelude::*, sprite::Anchor};

use crate::theme::FontAssets;

use super::{
    setup::{LevelProgress, Run},
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
        update_ui.run_if(hud_changed.and_then(in_state(Phase::Playing))),
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
/// The HUD shows the run and the progress; the RNG changes on every bounce, so it does not
/// count ([028]).
fn hud_changed(run: Res<Run>, progress: Res<LevelProgress>) -> bool {
    run.is_changed() || progress.is_changed()
}

fn setup_ui(
    mut commands: Commands,
    run: Res<Run>,
    progress: Res<LevelProgress>,
    fonts: Res<FontAssets>,
) {
    let font = fonts.main.clone();
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
            format!("Level: {}", run.level),
            SMALL_FONT_SIZE,
            Anchor::TOP_LEFT,
            Vec2::new(-side, top),
        ),
        UiLevel,
    ));
    commands.spawn((
        hud(
            run.points.to_string(),
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
                (progress.current * 100.) as u8,
                (progress.target * 100.) as u8
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
    run: Res<Run>,
    progress: Res<LevelProgress>,
) {
    if let Ok(mut text) = level_q.single_mut() {
        **text = format!("Level: {}", run.level);
    }

    if let Ok(mut text) = points_q.single_mut() {
        **text = run.points.to_string();
    }

    if let Ok(mut text) = progress_q.single_mut() {
        **text = format!(
            "{}/{}%",
            (progress.current * 100.) as u8,
            (progress.target * 100.) as u8
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::setup::GameRng;

    #[derive(Resource, Default)]
    struct Updates(u32);

    fn count(mut updates: ResMut<Updates>) {
        updates.0 += 1;
    }

    fn bounce(mut rng: ResMut<GameRng>) {
        rng.f32();
    }

    #[test]
    fn hud_updates_only_when_run_or_progress_change() {
        let mut app = App::new();
        app.insert_resource(Run::with_seed(1))
            .init_resource::<LevelProgress>()
            .insert_resource(GameRng::new(1))
            .init_resource::<Updates>()
            .add_systems(Update, (bounce, count.run_if(hud_changed)).chain());
        app.update();
        let first = app.world().resource::<Updates>().0;
        for _ in 0..600 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<Updates>().0,
            first,
            "10 s of RNG draws do not redraw the HUD"
        );
        app.world_mut().resource_mut::<Run>().points.0 += 50;
        app.update();
        app.world_mut().resource_mut::<LevelProgress>().current = 0.5;
        app.update();
        assert_eq!(app.world().resource::<Updates>().0, first + 2);
    }
}
