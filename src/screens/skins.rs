//! The skins screen ([078], D24, [114]): a preview of the chosen wall, fill and line, a
//! button per part that steps to the next unlocked skin (KILL plays its effect), the next
//! locked skin of each part with what it needs, and BACK.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{
        effects::{PixelExplosionMaterial, pixel_explosion},
        polygon::{LINE_THICKNESS, line_segment},
        skins::{
            self, FillSkin, KillSkin, LineSkin, MarkCursor, SkinAssets, Skins, Unlock, WallSkin,
        },
        wall::WALL_THICKNESS,
    },
    save::Save,
    shape::{self, Shape},
    theme::{
        FontAssets,
        palette::{BASE_GRAY, BURGUNDY, CYAN, NEON_WHITE},
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
/// The fill between the wall and the line, as a claim against a wall.
const FILL_Y: f32 = (WALL_Y - WALL_THICKNESS / 2. + LINE_Y) / 2.;
const PREVIEW_WIDTH: f32 = 340.;
/// Short pieces, as a finger draws them, so the laser's pulses run along ([093]).
const PREVIEW_PIECES: usize = 34;
const BUTTONS_Y: [f32; 4] = [118., 46., -26., -98.];
const BUTTON: Vec2 = Vec2::new(380., 62.);
const LOCKS_Y: f32 = -152.;
const LOCK_GAP: f32 = 28.;
const BEST_Y: f32 = -272.;
const BACK_Y: f32 = -340.;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum Part {
    Line,
    Wall,
    Fill,
    Kill,
}

impl Part {
    const ALL: [Self; 4] = [Self::Line, Self::Wall, Self::Fill, Self::Kill];

    fn label(self, skins: &Skins) -> String {
        match self {
            Part::Line => format!("LINE: {}", skins.line.name()),
            Part::Wall => format!("WALLS: {}", skins.wall.name()),
            Part::Fill => format!("FILL: {}", skins.fill.name()),
            Part::Kill => format!("KILL: {}", skins.kill.name()),
        }
    }

    fn step(self, save: &mut Save) {
        let (old, open) = (save.skins, |unlock: Unlock| unlock.open(save));
        let new = match self {
            Part::Line => Skins {
                line: skins::next(&LineSkin::ALL, old.line, |s| open(s.unlock())),
                ..old
            },
            Part::Wall => Skins {
                wall: skins::next(&WallSkin::ALL, old.wall, |s| open(s.unlock())),
                ..old
            },
            Part::Fill => Skins {
                fill: skins::next(&FillSkin::ALL, old.fill, |s| open(s.unlock())),
                ..old
            },
            Part::Kill => Skins {
                kill: skins::next(&KillSkin::ALL, old.kill, |s| open(s.unlock())),
                ..old
            },
        };
        save.skins = new;
    }
}

/// One row per part with a locked skin: its next one and what it needs.
fn lock_rows(save: &Save) -> Vec<String> {
    let all = skins::all_unlocks();
    ["LINE", "WALLS", "FILL", "KILL"]
        .into_iter()
        .filter_map(|part| {
            let (_, name, unlock) = all
                .iter()
                .find(|(p, _, unlock)| *p == part && !unlock.open(save))?;
            Some(format!("{name} {part}: {}", unlock.need(save)))
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
    for (part, y) in Part::ALL.into_iter().zip(BUTTONS_Y) {
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
    for (row, line) in lock_rows(&save).into_iter().enumerate() {
        commands.spawn(text(font, line, 20., LOCKS_Y - row as f32 * LOCK_GAP));
    }
    let best = format!(
        "BEST LEVEL {}   GAMES {}",
        save.best_level,
        save.stats.games()
    );
    commands.spawn(text(font, best, 22., BEST_Y));
    commands
        .spawn((
            DespawnOnExit(Screen::Skins),
            widget::button("BACK", font.clone(), BACK_Y),
        ))
        .observe(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Title);
        });
}

/// A KILL step plays the new kill on the preview.
fn step(
    activate: On<Activate>,
    mut commands: Commands,
    parts: Query<&Part>,
    mut save: ResMut<Save>,
    assets: Res<SkinAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosions: ResMut<Assets<PixelExplosionMaterial>>,
    time: Res<Time>,
) {
    let Ok(&part) = parts.get(activate.entity) else {
        return;
    };
    part.step(&mut save);
    if part != Part::Kill {
        return;
    }
    let at = Vec2::new(0., FILL_Y);
    match save.skins.kill {
        KillSkin::Pixel => {
            commands.spawn(pixel_explosion(
                &mut meshes,
                &mut explosions,
                time.elapsed_secs(),
                at,
                0.5,
            ));
        }
        skin => skins::spawn_kill(&mut commands, skin, &assets, at, BURGUNDY),
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

/// The wall, the fill and the line in the chosen skins, drawn as the game draws them.
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

    let half = PREVIEW_WIDTH / 2.;
    let (top, bottom) = (WALL_Y - WALL_THICKNESS / 2., LINE_Y);
    let corners = [
        Vec2::new(-half, bottom),
        Vec2::new(half, bottom),
        Vec2::new(half, top),
        Vec2::new(-half, top),
    ];
    let mut fill = commands.spawn((
        Preview,
        DespawnOnExit(Screen::Skins),
        Shape::new(shape::polygon(&corners)).fill(CYAN),
    ));
    if let Some(material) = assets.fill(save.skins.fill) {
        fill.insert(MeshMaterial2d(material));
    }

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

        save.stats.games = Some(30);
        Part::Fill.step(&mut save);
        Part::Kill.step(&mut save);
        assert_eq!(
            (save.skins.fill, save.skins.kill),
            (FillSkin::Scanlines, KillSkin::Shockwave)
        );
        Part::Fill.step(&mut save);
        assert_eq!(save.skins.fill, FillSkin::Solid, "grid needs 100 games");
        assert_eq!(Part::Kill.label(&save.skins), "KILL: SHOCKWAVE");
    }

    #[test]
    fn the_rows_show_the_next_lock_of_each_part() {
        let mut save = Save {
            best_level: 10,
            ..default()
        };
        save.stats.games = Some(18);
        assert_eq!(
            lock_rows(&save),
            [
                "CHAIN LINE: LEVEL 15",
                "CIRCUIT WALLS: LEVEL 20",
                "GRID FILL: 18 / 100 GAMES",
                "SHOCKWAVE KILL: 18 / 25 GAMES"
            ]
        );
        save.best_level = 20;
        save.stats.games = Some(1000);
        assert_eq!(lock_rows(&save), Vec::<String>::new(), "all open");
    }

    #[test]
    fn the_rows_fit_between_the_buttons_and_the_best_line() {
        let last_button = BUTTONS_Y[3] - BUTTON.y / 2.;
        let last_row = LOCKS_Y - (Part::ALL.len() - 1) as f32 * LOCK_GAP;
        assert!(LOCKS_Y + 12. < last_button && last_row - 12. > BEST_Y + 12.);
    }
}
