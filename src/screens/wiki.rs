//! The wiki ([071]): one row per enemy and item kind, with its look, name and what it
//! does, and for an enemy how many the player has eaten ([118]). A kind the player has not
//! met yet (`Save::met`, [070]) shows locked.

use bevy::{prelude::*, sprite::Anchor};

use super::Screen;
use crate::{
    game::{
        effects::GlowMaterial,
        enemies::{arrow, ball, rng::EnemyKind, shooter, splitter, square},
        items::{Item, auto_line, basic_bomb, multiplier, star, time_stop, token},
        setup::Points,
        tutorial::{KINDS, Kind},
    },
    save::Save,
    theme::{
        FontAssets,
        palette::{NEON_GRAY, NEON_WHITE, NEON_YELLOW},
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Wiki), setup);
}

const TITLE_Y: f32 = 330.;
const FIRST_Y: f32 = 255.;
const ROW_GAP: f32 = 58.;
const LOOK_X: f32 = -150.;
const TEXT_X: f32 = -100.;
/// The right end of an enemy's kill count; its stars sit after it.
const EATEN_X: f32 = 178.;
const MARK_GAP: f32 = 15.;
const MARK_SCALE: f32 = 0.45;
const BACK_Y: f32 = -340.;
/// The bomb's glow mesh is 100 wide; in a row it is half that.
const BOMB_SCALE: f32 = 0.5;

/// The name and the text of a row.
fn row_text(kind: Kind, met: bool) -> (&'static str, String) {
    if met {
        (kind.name(), kind.about())
    } else {
        ("???", "NOT MET YET".into())
    }
}

/// An enemy's kill count, its color and its marks: a star at 100 and two at 1000 ([118]).
fn eaten_text(eaten: u64) -> (String, Color, usize) {
    let text = format!("EATEN {}", Points(eaten.into()));
    match eaten {
        0..100 => (text, NEON_GRAY, 0),
        100..1000 => (text, NEON_YELLOW, 1),
        _ => (text, NEON_YELLOW, 2),
    }
}

fn label(font: &Handle<Font>, text: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text2d::new(text),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(NEON_WHITE),
    )
}

fn setup(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    save: Res<Save>,
    item_assets: Res<basic_bomb::ItemAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut colors: ResMut<Assets<ColorMaterial>>,
    mut glows: ResMut<Assets<GlowMaterial>>,
) {
    let font = &fonts.main;
    commands.spawn((
        DespawnOnExit(Screen::Wiki),
        label(font, "WIKI", 54.),
        Transform::from_xyz(0., TITLE_Y, 1.),
    ));
    for (row, kind) in KINDS.into_iter().enumerate() {
        let y = FIRST_Y - row as f32 * ROW_GAP;
        let met = save.met.contains(&kind.id());
        let mut look = commands.spawn((
            DespawnOnExit(Screen::Wiki),
            Transform::from_xyz(LOOK_X, y, 1.),
        ));
        match (met, kind) {
            (false, _) => look.insert(label(font, "?", 40.)),
            (true, Kind::Enemy(EnemyKind::Ball)) => {
                look.insert(ball::look(&mut meshes, &mut colors))
            }
            (true, Kind::Enemy(EnemyKind::Square)) => {
                look.insert(square::look(&mut meshes, &mut colors))
            }
            (true, Kind::Enemy(EnemyKind::Arrow)) => look.insert(arrow::look()),
            (true, Kind::Enemy(EnemyKind::Splitter)) => {
                look.insert(splitter::look(splitter::Splitter::Queen))
            }
            (true, Kind::Enemy(EnemyKind::Shooter)) => look.insert(shooter::look()),
            (true, Kind::Item(Item::BasicBomb)) => look.insert((
                basic_bomb::look(&item_assets, &mut meshes, &mut glows),
                Transform::from_xyz(LOOK_X, y, 1.).with_scale(Vec3::splat(BOMB_SCALE)),
            )),
            (true, Kind::Item(Item::Multiplier)) => look.insert(token(multiplier::icon())),
            (true, Kind::Item(Item::TimeStop)) => look.insert(token(time_stop::icon())),
            (true, Kind::Item(Item::AutoLine)) => look.insert(token(auto_line::icon())),
            (true, Kind::Item(Item::Star)) => look.insert(token(star::icon())),
        };
        if let (true, Kind::Enemy(_)) = (met, kind) {
            let eaten = save.stats.eaten.get(&kind.id()).copied().unwrap_or(0);
            let (text, color, marks) = eaten_text(eaten);
            for mark in 0..marks {
                let mut star = star::icon();
                star.fill = Some(NEON_YELLOW);
                star.stroke = None;
                let x = EATEN_X + MARK_GAP * (mark as f32 + 0.8);
                commands.spawn((
                    DespawnOnExit(Screen::Wiki),
                    star,
                    Transform::from_xyz(x, y + 11., 1.).with_scale(Vec3::splat(MARK_SCALE)),
                ));
            }
            commands
                .spawn((
                    DespawnOnExit(Screen::Wiki),
                    label(font, text, 16.),
                    Anchor::CENTER_RIGHT,
                    Transform::from_xyz(EATEN_X, y + 11., 1.),
                ))
                .insert(TextColor(color));
        }
        let (name, about) = row_text(kind, met);
        commands.spawn((
            DespawnOnExit(Screen::Wiki),
            label(font, name, 24.),
            Anchor::CENTER_LEFT,
            Transform::from_xyz(TEXT_X, y + 11., 1.),
        ));
        commands.spawn((
            DespawnOnExit(Screen::Wiki),
            label(font, about, 16.),
            Anchor::CENTER_LEFT,
            Transform::from_xyz(TEXT_X, y - 13., 1.),
        ));
    }
    commands
        .spawn((
            DespawnOnExit(Screen::Wiki),
            widget::button("BACK", font.clone(), BACK_Y),
        ))
        .observe(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Title);
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{enemies::rng::SPAWN_TABLE, items::ITEM_LEVELS};

    #[test]
    fn a_kill_count_gets_a_mark_at_100_and_at_1000() {
        assert_eq!(eaten_text(0), ("EATEN 0".into(), NEON_GRAY, 0));
        assert_eq!(eaten_text(99).2, 0);
        assert_eq!(eaten_text(100), ("EATEN 100".into(), NEON_YELLOW, 1));
        assert_eq!(eaten_text(1_204), ("EATEN 1,204".into(), NEON_YELLOW, 2));
        const {
            assert!(
                EATEN_X + MARK_GAP * 1.8 + 14. * MARK_SCALE < 225.,
                "two stars fit"
            )
        };
    }

    #[test]
    fn every_spawned_kind_has_a_row_with_a_name_and_a_text() {
        for rule in SPAWN_TABLE {
            assert!(KINDS.contains(&Kind::Enemy(rule.kind)), "{:?}", rule.kind);
        }
        for (item, _) in ITEM_LEVELS {
            assert!(KINDS.contains(&Kind::Item(*item)), "{item:?}");
        }
        for kind in KINDS {
            let (name, about) = row_text(kind, true);
            assert!(!name.is_empty() && !about.is_empty(), "{kind:?}");
            assert!(about.len() <= 24, "{about} fits the row");
        }
    }

    #[test]
    fn a_kind_not_met_shows_locked() {
        let (name, about) = row_text(KINDS[0], false);
        assert_eq!((name, about.as_str()), ("???", "NOT MET YET"));
    }
}
