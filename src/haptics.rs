//! Touch feedback on phones ([081]): a light pulse for a claim, a medium one for a kill or a
//! bomb, a heavy one for the death. Android calls `MainActivity.haptic` over JNI; iOS uses
//! `UIImpactFeedbackGenerator`. Desktop and web only log the pulse.

use bevy::{ecs::system::NonSendMarker, prelude::*};

use crate::{audio::Sfx, game::state::IsPaused, save::Save};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Pending>()
        .add_observer(on_sfx)
        .add_systems(OnEnter(IsPaused::Dying), |mut pending: ResMut<Pending>| {
            pending.ask(Haptic::Heavy);
        })
        .add_systems(Update, pulse);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Haptic {
    Light,
    Medium,
    Heavy,
}

/// The strongest pulse asked for since the last frame: the kills of one claim buzz once.
#[derive(Resource, Default)]
struct Pending(Option<Haptic>);

impl Pending {
    fn ask(&mut self, kind: Haptic) {
        self.0 = self.0.max(Some(kind));
    }
}

fn on_sfx(sfx: On<Sfx>, mut pending: ResMut<Pending>) {
    match *sfx {
        // A countdown's last seconds tick on the hand too ([122]).
        Sfx::Claim | Sfx::Tick => pending.ask(Haptic::Light),
        Sfx::Eat | Sfx::Bomb => pending.ask(Haptic::Medium),
        Sfx::Bounce
        | Sfx::Shot
        | Sfx::Hatch
        | Sfx::Multiplier
        | Sfx::TimeStop
        | Sfx::AutoLine
        | Sfx::Star => {}
    }
}

/// On the main thread: UIKit only works there. The settings can turn it off ([048]).
fn pulse(_: NonSendMarker, mut pending: ResMut<Pending>, save: Res<Save>) {
    let Some(kind) = pending.0.take() else {
        return;
    };
    if !save.settings.haptics {
        return;
    }
    info!("haptic {kind:?}");
    play(kind);
}

#[cfg(target_os = "android")]
fn play(kind: Haptic) {
    use jni::{
        JavaVM,
        objects::{JObject, JValue},
    };

    let Some(app) = bevy::android::ANDROID_APP.get() else {
        return;
    };
    let result = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.and_then(|vm| {
        let mut env = vm.attach_current_thread_permanently()?;
        let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
        let called = env.call_method(&activity, "haptic", "(I)V", &[JValue::Int(kind as i32)]);
        if called.is_err() {
            env.exception_clear()?;
        }
        called.map(drop)
    });
    if let Err(err) = result {
        warn!("haptic {kind:?} failed: {err}");
    }
}

#[cfg(target_os = "ios")]
fn play(kind: Haptic) {
    use objc2_foundation::MainThreadMarker;
    use objc2_ui_kit::{UIImpactFeedbackGenerator, UIImpactFeedbackStyle};

    let Some(mtm) = MainThreadMarker::new() else {
        warn!("haptic {kind:?}: not on the main thread");
        return;
    };
    let style = match kind {
        Haptic::Light => UIImpactFeedbackStyle::Light,
        Haptic::Medium => UIImpactFeedbackStyle::Medium,
        Haptic::Heavy => UIImpactFeedbackStyle::Heavy,
    };
    // `initWithStyle:` is deprecated since iOS 17 for `feedbackGeneratorWithStyle:forView:`;
    // it still works and needs no view.
    #[allow(deprecated)]
    let generator = unsafe { UIImpactFeedbackGenerator::initWithStyle(mtm.alloc(), style) };
    unsafe { generator.impactOccurred() };
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn play(_: Haptic) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn pulse_for(sounds: &[Sfx]) -> Option<Haptic> {
        let mut world = World::new();
        world.init_resource::<Pending>();
        world.add_observer(on_sfx);
        for &sfx in sounds {
            world.trigger(sfx);
        }
        world.flush();
        world.resource::<Pending>().0
    }

    #[test]
    fn the_strongest_pulse_of_a_frame_wins_and_a_bounce_buzzes_none() {
        assert_eq!(pulse_for(&[Sfx::Claim]), Some(Haptic::Light));
        assert_eq!(
            pulse_for(&[Sfx::Claim, Sfx::Eat, Sfx::Eat, Sfx::Claim]),
            Some(Haptic::Medium)
        );
        assert_eq!(pulse_for(&[Sfx::Bomb]), Some(Haptic::Medium));
        assert_eq!(pulse_for(&[Sfx::Bounce]), None);
        let mut pending = Pending(Some(Haptic::Medium));
        pending.ask(Haptic::Heavy);
        pending.ask(Haptic::Light);
        assert_eq!(pending.0, Some(Haptic::Heavy));
    }
}
