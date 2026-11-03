//! Death ([038]): the run freezes and the line that was hit flashes, then the game over
//! screen shows. On phones the frame of the death is kept for the share sheet ([095]).

use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};

use super::{
    pause::{start_time, stop_time},
    polygon::TempLineSegment,
    state::IsPaused,
};
use crate::{
    screens::Screen,
    share::SHEET,
    theme::palette::{NEON_BURGUNDY, NEON_WHITE},
};

const DEATH_SECS: f32 = 1.;
const FLASH_SECS: f32 = 0.1;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(IsPaused::Dying), (stop_time, start_death))
        .add_systems(OnExit(IsPaused::Dying), start_time)
        .add_systems(Update, flash_line.run_if(in_state(IsPaused::Dying)));
    if SHEET {
        app.add_systems(OnEnter(IsPaused::Dying), capture);
    }
}

/// The frame of the last death, for the share sheet ([095]).
#[derive(Resource)]
pub struct DeathPicture(pub Image);

fn capture(mut commands: Commands) {
    commands.remove_resource::<DeathPicture>();
    commands.spawn(Screenshot::primary_window()).observe(
        |shot: On<ScreenshotCaptured>, mut commands: Commands| {
            commands.insert_resource(DeathPicture(shot.image.clone()));
        },
    );
}

/// Real time: virtual time stands still while dying.
#[derive(Resource)]
struct DeathTimer(Timer);

fn start_death(mut commands: Commands) {
    commands.insert_resource(DeathTimer(Timer::from_seconds(DEATH_SECS, TimerMode::Once)));
}

fn flash_line(
    time: Res<Time<Real>>,
    mut timer: ResMut<DeathTimer>,
    mut line: Query<&mut Sprite, With<TempLineSegment>>,
    mut next: ResMut<NextState<Screen>>,
) {
    timer.0.tick(time.delta());
    let red = ((timer.0.elapsed_secs() / FLASH_SECS) as u32).is_multiple_of(2);
    for mut sprite in &mut line {
        sprite.color = if red { NEON_BURGUNDY } else { NEON_WHITE };
    }
    if timer.0.is_finished() {
        next.set(Screen::GameOver);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use avian2d::prelude::Physics;
    use bevy::{state::app::StatesPlugin, time::TimeUpdateStrategy};

    use super::*;
    use crate::game::state::Phase;

    #[test]
    fn a_death_freezes_time_flashes_the_line_then_shows_game_over() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1. / 60.,
            )))
            .init_resource::<Time<Physics>>()
            .init_state::<Screen>()
            .add_sub_state::<Phase>()
            .add_sub_state::<IsPaused>()
            .insert_state(Screen::Gameplay)
            .add_plugins(plugin);
        let line = app
            .world_mut()
            .spawn((TempLineSegment, Sprite::default()))
            .id();
        app.update();
        let color = |app: &App| app.world().get::<Sprite>(line).unwrap().color;
        let screen = |app: &App| *app.world().resource::<State<Screen>>().get();

        app.world_mut()
            .resource_mut::<NextState<IsPaused>>()
            .set(IsPaused::Dying);
        app.update();
        assert_eq!(color(&app), NEON_BURGUNDY);
        assert!(app.world().resource::<Time<Virtual>>().is_paused());
        let mut colors = vec![];
        let mut frames = 1;
        while screen(&app) != Screen::GameOver {
            app.update();
            colors.push(color(&app));
            frames += 1;
            assert!(frames < 70, "no game over after {frames} frames");
        }
        assert!(frames >= 60, "the death lasts 1 s: {frames} frames");
        assert!(colors.contains(&NEON_WHITE), "the line flashes");
        assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    }
}
