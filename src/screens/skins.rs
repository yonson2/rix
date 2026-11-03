//! The skins screen ([078], D24): a preview of the chosen wall and line skins, a button per
//! part that steps to the next unlocked skin, the level that unlocks each skin, and BACK.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{
        polygon::{LINE_THICKNESS, line_segment},
        skins::{self, LineSkin, MarkCursor, SkinAssets, Skins, WallSkin},
        wall::WALL_THICKNESS,
    },
    save::Save,
    theme::{
        FontAssets,
        palette::{BASE_GRAY, NEON_WHITE},
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Skins), (setup, spawn_preview))
        .add_systems(
            Update,
            (relabel, (despawn_preview, spawn_preview).chain())
                .run_if(in_state(Screen::Skins))
                .run_if(resource_changed::<Save>),
        );
}

const TITLE_Y: f32 = 330.;
const WALL_Y: f32 = 250.;
const LINE_Y: f32 = 190.;
const PREVIEW_WIDTH: f32 = 340.;
/// Short pieces, as a finger draws them, so the laser's pulses run along ([093]).
const PREVIEW_PIECES: usize = 34;
const LINE_BUTTON_Y: f32 = 100.;
const WALL_BUTTON_Y: f32 = 10.;
const BUTTON: Vec2 = Vec2::new(380., 76.);
const LOCKS_Y: f32 = -80.;
const LOCK_GAP: f32 = 34.;
const BEST_Y: f32 = -245.;
const BACK_Y: f32 = -340.;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum Part {
    Line,
    Wall,
}

impl Part {
    fn label(self, skins: &Skins) -> String {
        match self {
            Part::Line => format!("LINE: {}", skins.line.name()),
            Part::Wall => format!("WALLS: {}", skins.wall.name()),
        }
    }

    fn step(self, save: &mut Save) {
        let best = save.best_level;
        match self {
            Part::Line => {
                save.skins.line =
                    skins::next(&LineSkin::ALL, save.skins.line, |s| s.level() <= best)
            }
            Part::Wall => {
                save.skins.wall =
                    skins::next(&WallSkin::ALL, save.skins.wall, |s| s.level() <= best)
            }
        }
    }
}

/// One row per skin to unlock, in level order: what it is and the level that unlocks it.
fn lock_rows(best: usize) -> Vec<String> {
    let line = LineSkin::ALL[1..]
        .iter()
        .map(|s| (s.level(), format!("{} LINE", s.name())));
    let wall = WallSkin::ALL[1..]
        .iter()
        .map(|s| (s.level(), format!("{} WALLS", s.name())));
    let mut rows: Vec<(usize, String)> = line.chain(wall).collect();
    rows.sort();
    rows.into_iter()
        .map(|(level, name)| match level <= best {
            true => format!("{name}: UNLOCKED"),
            false => format!("{name}: LEVEL {level}"),
        })
        .collect()
}

fn text(font: &Handle<Font>, text: impl Into<String>, size: f32, y: f32) -> impl Bundle {
    (
        DespawnOnExit(Screen::Skins),
        Text2d::new(text),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(NEON_WHITE),
        Transform::from_xyz(0., y, 1.),
    )
}

fn setup(mut commands: Commands, fonts: Res<FontAssets>, save: Res<Save>) {
    let font = &fonts.main;
    commands.spawn(text(font, "SKINS", 54., TITLE_Y));
    for (part, y) in [(Part::Line, LINE_BUTTON_Y), (Part::Wall, WALL_BUTTON_Y)] {
        commands
            .spawn((
                DespawnOnExit(Screen::Skins),
                part,
                widget::button_at(
                    &part.label(&save.skins),
                    font.clone(),
                    Vec2::new(0., y),
                    BUTTON,
                ),
            ))
            .observe(step);
    }
    for (row, line) in lock_rows(save.best_level).into_iter().enumerate() {
        commands.spawn(text(font, line, 20., LOCKS_Y - row as f32 * LOCK_GAP));
    }
    commands.spawn(text(
        font,
        format!("BEST LEVEL {}", save.best_level),
        22.,
        BEST_Y,
    ));
    commands
        .spawn((
            DespawnOnExit(Screen::Skins),
            widget::button("BACK", font.clone(), BACK_Y),
        ))
        .observe(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Title);
        });
}

fn step(activate: On<Activate>, parts: Query<&Part>, mut save: ResMut<Save>) {
    if let Ok(part) = parts.get(activate.entity) {
        part.step(&mut save);
    }
}

fn relabel(save: Res<Save>, parts: Query<(&Part, &Children)>, mut labels: Query<&mut Text2d>) {
    for (part, children) in &parts {
        for child in children {
            if let Ok(mut label) = labels.get_mut(*child) {
                label.0 = part.label(&save.skins);
            }
        }
    }
}

/// The wall and the line in the chosen skins, drawn as the game draws them.
#[derive(Component)]
struct Preview;

fn spawn_preview(
    mut commands: Commands,
    save: Res<Save>,
    assets: Res<SkinAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut wall = commands.spawn((
        Preview,
        DespawnOnExit(Screen::Skins),
        Transform::from_xyz(0., WALL_Y, 1.),
    ));
    skins::insert_wall_look(
        &mut wall,
        save.skins.wall,
        &assets,
        Vec2::new(PREVIEW_WIDTH, WALL_THICKNESS),
        &mut meshes,
        &materials.add(BASE_GRAY),
    );

    let skin = save.skins.line;
    let mut cursor = MarkCursor::default();
    let points: Vec<Vec2> = (0..=PREVIEW_PIECES)
        .map(|i| {
            Vec2::new(
                PREVIEW_WIDTH * (i as f32 / PREVIEW_PIECES as f32 - 0.5),
                LINE_Y,
            )
        })
        .collect();
    for ends in points.windows(2) {
        let mut segment = commands.spawn((
            Preview,
            DespawnOnExit(Screen::Skins),
            line_segment(ends[0], ends[1], LINE_THICKNESS, skin.core().0),
        ));
        skins::add_marks(&mut segment, skin, &assets, (ends[0], ends[1]), &mut cursor);
    }
}

fn despawn_preview(mut commands: Commands, previews: Query<Entity, With<Preview>>) {
    for entity in &previews {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_part_steps_only_through_unlocked_skins() {
        let mut save = Save::default();
        Part::Line.step(&mut save);
        Part::Wall.step(&mut save);
        assert_eq!(
            save.skins,
            Skins::default(),
            "a fresh save has nothing to step to"
        );

        save.best_level = 12;
        Part::Line.step(&mut save);
        Part::Wall.step(&mut save);
        assert_eq!(
            (save.skins.line, save.skins.wall),
            (LineSkin::Laser, WallSkin::Brick)
        );
        Part::Line.step(&mut save);
        assert_eq!(save.skins.line, LineSkin::Neon, "chain needs level 15");
        assert_eq!(Part::Wall.label(&save.skins), "WALLS: BRICK");
    }

    #[test]
    fn the_rows_show_each_skin_and_its_level() {
        assert_eq!(
            lock_rows(10),
            [
                "LASER LINE: UNLOCKED",
                "BRICK WALLS: UNLOCKED",
                "CHAIN LINE: LEVEL 15",
                "CIRCUIT WALLS: LEVEL 20"
            ]
        );
    }
}
