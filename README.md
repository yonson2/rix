# Rix

Rix is an arcade game for phones, after Qix. You draw a line from wall to wall with one
finger. The line cuts the field: the side without enemies becomes yours. An enemy that
touches the line while you draw it ends the run. Claim 75 % of the field to clear a level.
Each level brings more enemies and items, and points double per level. The look is neon on
dark, with bloom and a CRT screen.

Rix is written in Rust with the [Bevy](https://bevy.org) engine. It is made for iOS and
Android first; a web build for itch.io comes after.

## Inspirations

- **Qix** (Taito, 1981): claim the field with lines while enemies roam it.
- **Xonix** (1984): the same idea on a grid, with enemies that bounce.
- **Geometry Wars** and **Hyper Light Drifter**: the neon look.

## Build

You need Rust 1.97.1 or newer ([rustup](https://rustup.rs)) and
[just](https://github.com/casey/just).

```sh
just dev        # run on the desktop (a dev build)
cargo test      # the tests
just shot       # play a scripted game without a window and save a screenshot
```

### Android

You need Android Studio (its SDK, NDK and JDK 21) and
[cargo-ndk](https://github.com/bbqsrc/cargo-ndk):

```sh
rustup target add aarch64-linux-android
cargo install cargo-ndk
just android        # build the debug APK
just android-run    # install it and start it on the connected phone or emulator
```

### iOS

You need Xcode on a Mac:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
just ios-sim        # build, install and start on the iPhone simulator
```

## More

- `AGENTS.md`: how work on Rix is done, and every command.
- `ARCHITECTURE.md`: the code map.
- `issues/`: the issue tracker.
- `CREDITS.md`: who made what, and the licences.
- `PRIVACY.md`: the privacy policy. Rix collects no data.
