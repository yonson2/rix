//! Share and paste ([067]). On phones: the share sheet and the system clipboard, over JNI to
//! `MainActivity` on Android and with UIKit on iOS. Elsewhere: Bevy's clipboard (a share is
//! a copy of the text). UIKit works only on the main thread, so `run` (`NonSendMarker`) does
//! the work. A share's picture ([095]) goes to the sheet as a PNG file.

use bevy::{
    clipboard::{Clipboard, ClipboardRead},
    ecs::system::NonSendMarker,
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Requests>()
        .add_observer(|share: On<Share>, mut requests: ResMut<Requests>| {
            requests.share = Some((share.text.clone(), share.picture.clone()));
        })
        .add_observer(|_: On<Paste>, mut requests: ResMut<Requests>| {
            requests.paste = true;
        })
        .add_systems(Update, run);
}

/// Phones share through the share sheet; elsewhere a share copies the text.
pub const SHEET: bool = cfg!(any(target_os = "android", target_os = "ios"));

/// Shares a text and a picture: the share sheet on phones, a copy of the text to the
/// clipboard elsewhere.
#[derive(Event)]
pub struct Share {
    pub text: String,
    pub picture: Option<Image>,
}

/// Asks for the clipboard's text; [`Pasted`] answers.
#[derive(Event)]
pub struct Paste;

#[derive(Event)]
pub struct Pasted(pub String);

#[derive(Resource, Default)]
struct Requests {
    share: Option<(String, Option<Image>)>,
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
    if let Some((text, picture)) = requests.share.take() {
        share(&text, picture, &mut clipboard);
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
fn share(text: &str, _: Option<Image>, clipboard: &mut Clipboard) {
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

/// Writes `picture` to `path` as a PNG for the share sheet.
#[cfg(any(target_os = "android", target_os = "ios"))]
fn write_png(picture: Image, path: std::path::PathBuf) -> Option<std::path::PathBuf> {
    let result = picture
        .try_into_dynamic()
        .map_err(|err| err.to_string())
        .and_then(|picture| {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
            }
            picture.to_rgb8().save(&path).map_err(|err| err.to_string())
        });
    match result {
        Ok(()) => Some(path),
        Err(err) => {
            warn!("share picture failed: {err}");
            None
        }
    }
}

/// The picture goes to the app's files directory, which `MainActivity`'s `FileProvider`
/// serves (`res/xml/share_paths.xml`).
#[cfg(target_os = "android")]
fn share(text: &str, picture: Option<Image>, _: &mut Clipboard) {
    use jni::objects::{JObject, JValue};

    let path = picture.and_then(|picture| {
        let dir = bevy::android::ANDROID_APP.get()?.internal_data_path()?;
        write_png(picture, dir.join("share/rix.png"))
    });
    let result = with_activity(|env, activity| {
        let text = env.new_string(text)?;
        let path = match &path {
            Some(path) => JObject::from(env.new_string(path.to_string_lossy())?),
            None => JObject::null(),
        };
        env.call_method(
            activity,
            "share",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[JValue::Object(&text), JValue::Object(&path)],
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
fn share(text: &str, picture: Option<Image>, _: &mut Clipboard) {
    use objc2::rc::Retained;
    use objc2_foundation::{MainThreadMarker, NSArray, NSString, NSURL};
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
    let mut items = vec![Retained::into_super(Retained::into_super(
        NSString::from_str(text),
    ))];
    let path = picture.and_then(|picture| write_png(picture, std::env::temp_dir().join("rix.png")));
    if let Some(path) = path {
        let path = NSString::from_str(&path.to_string_lossy());
        let url = unsafe { NSURL::fileURLWithPath(&path) };
        items.push(Retained::into_super(Retained::into_super(url)));
    }
    let items = NSArray::from_vec(items);
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
