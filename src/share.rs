//! Share and paste ([067]). On phones: the share sheet and the system clipboard, over JNI to
//! `MainActivity` on Android and with UIKit on iOS. Elsewhere: Bevy's clipboard (a share is
//! a copy). UIKit works only on the main thread, so `run` (`NonSendMarker`) does the work.

use bevy::{
    clipboard::{Clipboard, ClipboardRead},
    ecs::system::NonSendMarker,
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Requests>()
        .add_observer(|share: On<Share>, mut requests: ResMut<Requests>| {
            requests.share = Some(share.0.clone());
        })
        .add_observer(|_: On<Paste>, mut requests: ResMut<Requests>| {
            requests.paste = true;
        })
        .add_systems(Update, run);
}

/// Phones share through the share sheet; elsewhere a share copies the text.
pub const SHEET: bool = cfg!(any(target_os = "android", target_os = "ios"));

/// Shares a text: the share sheet on phones, a copy to the clipboard elsewhere.
#[derive(Event)]
pub struct Share(pub String);

/// Asks for the clipboard's text; [`Pasted`] answers.
#[derive(Event)]
pub struct Paste;

#[derive(Event)]
pub struct Pasted(pub String);

#[derive(Resource, Default)]
struct Requests {
    share: Option<String>,
    paste: bool,
}

/// A read can take frames on the web.
fn run(
    _: NonSendMarker,
    mut commands: Commands,
    mut requests: ResMut<Requests>,
    mut clipboard: ResMut<Clipboard>,
    mut reading: Local<Option<ClipboardRead>>,
) {
    if let Some(text) = requests.share.take() {
        share(&text, &mut clipboard);
    }
    if std::mem::take(&mut requests.paste) {
        *reading = Some(paste(&mut clipboard));
    }
    let Some(result) = reading.as_mut().and_then(ClipboardRead::poll_result) else {
        return;
    };
    *reading = None;
    match result {
        Ok(text) => commands.trigger(Pasted(text)),
        Err(err) => warn!("paste failed: {err}"),
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn share(text: &str, clipboard: &mut Clipboard) {
    if let Err(err) = clipboard.set_text(text.to_owned()) {
        warn!("copy failed: {err}");
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn paste(clipboard: &mut Clipboard) -> ClipboardRead {
    clipboard.fetch_text()
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn paste(_: &mut Clipboard) -> ClipboardRead {
    ClipboardRead::Ready(phone_paste().ok_or(bevy::clipboard::ClipboardError::ContentNotAvailable))
}

#[cfg(target_os = "android")]
fn share(text: &str, _: &mut Clipboard) {
    use jni::objects::JValue;

    let result = with_activity(|env, activity| {
        let text = env.new_string(text)?;
        env.call_method(
            activity,
            "share",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&text)],
        )
        .map(drop)
    });
    if let Err(err) = result {
        warn!("share failed: {err}");
    }
}

#[cfg(target_os = "android")]
fn phone_paste() -> Option<String> {
    use jni::objects::JString;

    let result = with_activity(|env, activity| {
        let text = env
            .call_method(activity, "pasteText", "()Ljava/lang/String;", &[])?
            .l()?;
        if text.is_null() {
            return Ok(None);
        }
        Ok(Some(env.get_string(&JString::from(text))?.into()))
    });
    result.unwrap_or_else(|err| {
        warn!("paste failed: {err}");
        None
    })
}

/// Calls `f` with the JNI environment and the activity; clears a Java exception it threw.
#[cfg(target_os = "android")]
fn with_activity<T>(
    f: impl FnOnce(&mut jni::JNIEnv, &jni::objects::JObject) -> jni::errors::Result<T>,
) -> jni::errors::Result<T> {
    use jni::{JavaVM, objects::JObject};

    let app = bevy::android::ANDROID_APP
        .get()
        .ok_or(jni::errors::Error::NullPtr("no Android app"))?;
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }?;
    let mut env = vm.attach_current_thread_permanently()?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    let result = f(&mut env, &activity);
    if result.is_err() {
        env.exception_clear()?;
    }
    result
}

#[cfg(target_os = "ios")]
fn share(text: &str, _: &mut Clipboard) {
    use objc2::rc::Retained;
    use objc2_foundation::{MainThreadMarker, NSArray, NSString};
    use objc2_ui_kit::{UIActivityViewController, UIApplication};

    let Some(mtm) = MainThreadMarker::new() else {
        warn!("share: not on the main thread");
        return;
    };
    // `keyWindow` is deprecated for apps with many scenes; Rix has one.
    #[allow(deprecated)]
    let root = unsafe { UIApplication::sharedApplication(mtm).keyWindow() }
        .and_then(|window| window.rootViewController());
    let Some(root) = root else {
        warn!("share: no root view controller");
        return;
    };
    let items = NSArray::from_vec(vec![Retained::into_super(Retained::into_super(
        NSString::from_str(text),
    ))]);
    unsafe {
        let sheet = UIActivityViewController::initWithActivityItems_applicationActivities(
            mtm.alloc(),
            &items,
            None,
        );
        // An iPad shows the sheet as a popover, which needs a source view.
        if let Some(popover) = sheet.popoverPresentationController() {
            popover.setSourceView(root.view().as_deref());
        }
        root.presentViewController_animated_completion(&sheet, true, None);
    }
}

#[cfg(target_os = "ios")]
fn phone_paste() -> Option<String> {
    use objc2_ui_kit::UIPasteboard;

    unsafe { UIPasteboard::generalPasteboard().string() }.map(|text| text.to_string())
}
