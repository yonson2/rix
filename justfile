default: dev
dev:
    cargo run --features dev

# Screenshot of a scripted run (one line across the field) into target/shots, then exit ([006]).
shot path="target/shots/game.png":
    mkdir -p "$(dirname '{{path}}')"
    RIX_SEED=1 RIX_SHOT='{{path}}' cargo run --features dev_tools

# Screenshot of the menu into target/shots, then exit ([006]).
shot-menu path="target/shots/menu.png":
    mkdir -p "$(dirname '{{path}}')"
    RIX_SEED=1 RIX_SHOT='{{path}}' RIX_SHOT_SCENE=menu cargo run --features dev_tools

# Scripted claim, level clear, death and retry; prints the entity census at each change ([026]).
shot-flow path="target/shots/flow.png":
    mkdir -p "$(dirname '{{path}}')"
    RIX_SEED=1 RIX_SHOT='{{path}}' RIX_SHOT_SCENE=flow cargo run --features dev_tools 2>&1 | grep -o 'RIX_FLOW.*'

# Replay test ([072], [075]): a seed with the scripted line ends in the same state hash at
# 30, 60 and 120 frames per second, without the renderer, and the hashes match
# tools/replay-hashes.txt. CI runs it on Linux x86_64 and macOS arm64.
replay:
    tools/replay.sh

android_sdk := env_var_or_default("ANDROID_HOME", home_directory() / "Library/Android/sdk")
android_jdk := env_var_or_default("JAVA_HOME", "/Applications/Android Studio.app/Contents/jbr/Contents/Home")
android_dir := "mobile/android"

# Android debug APK ([021]): the Rust library via cargo-ndk into jniLibs, then Gradle.
android:
    ANDROID_HOME='{{android_sdk}}' cargo ndk -t arm64-v8a --platform 31 -o {{android_dir}}/app/src/main/jniLibs build --package rix_mobile
    cd {{android_dir}} && ANDROID_HOME='{{android_sdk}}' JAVA_HOME='{{android_jdk}}' ./gradlew --quiet assembleDebug

# Android release bundle for Play ([054]): mobile/android/app/build/outputs/bundle/release/app-release.aab,
# signed with the upload key that `.env` names (RIX_UPLOAD_KEYSTORE, RIX_UPLOAD_PASSWORD).
# cargo-ndk leaves the library unstripped: Gradle strips it and keeps its symbols ([105]).
android-release:
    ANDROID_HOME='{{android_sdk}}' cargo ndk -t arm64-v8a --platform 31 --no-strip -o {{android_dir}}/app/src/main/jniLibs build --release --package rix_mobile
    set -a && . ./.env && set +a && cd {{android_dir}} && ANDROID_HOME='{{android_sdk}}' JAVA_HOME='{{android_jdk}}' ./gradlew --quiet bundleRelease

# Install the debug APK on the running emulator or device and start it.
android-run: android
    '{{android_sdk}}/platform-tools/adb' install -r {{android_dir}}/app/build/outputs/apk/debug/app-debug.apk
    '{{android_sdk}}/platform-tools/adb' shell am start -n top.cosas.rix/.MainActivity

ios_dir := "mobile/ios"
ios_sim := env_var_or_default("IOS_SIM", "iPhone 16")
# The app version is the Cargo.toml version (D12); the build number is
# major * 10000 + minor * 100 + patch, as on Android.
version := `grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2`
build_number := `grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2 | awk -F. '{print $1 * 10000 + $2 * 100 + $3}'`
ios_build := "xcodebuild -quiet -project " + ios_dir + "/Rix.xcodeproj -scheme Rix -configuration Debug -derivedDataPath target/ios MARKETING_VERSION=" + version + " CURRENT_PROJECT_VERSION=" + build_number

# Build, install and start on the iOS simulator `IOS_SIM` (default iPhone 16), no window ([020]).
ios-sim:
    #!/usr/bin/env bash
    set -euo pipefail
    id=$(xcrun simctl list devices available | grep -F "    {{ios_sim}} (" | tail -1 | grep -Eo '[0-9A-F-]{36}')
    {{ios_build}} -destination "id=$id"
    xcrun simctl boot "$id" 2>/dev/null || true
    xcrun simctl install "$id" target/ios/Build/Products/Debug-iphonesimulator/Rix.app
    xcrun simctl launch "$id" top.cosas.rix

# Release archive for the App Store ([053]): target/ios/Rix.xcarchive, signed by Pedro's team.
ios-archive:
    {{replace(ios_build, "Debug", "Release")}} archive -destination generic/platform=iOS -archivePath target/ios/Rix.xcarchive -allowProvisioningUpdates

# Upload the archive to App Store Connect (TestFlight) with the Apple account in Xcode ([053]).
# /usr/bin first: the export runs /usr/bin/rsync, which fails if it finds Homebrew's rsync.
ios-upload: ios-archive
    PATH="/usr/bin:$PATH" xcodebuild -quiet -exportArchive -archivePath target/ios/Rix.xcarchive -exportOptionsPlist {{ios_dir}}/ExportOptions.plist -exportPath target/ios/export -allowProvisioningUpdates

# Build (signed by Pedro's team), install and start on iPhone `IOS_DEVICE` (`xcrun devicectl list devices`).
ios-device:
    #!/usr/bin/env bash
    set -euo pipefail
    : "${IOS_DEVICE:?set IOS_DEVICE (xcrun devicectl list devices)}"
    {{ios_build}} -destination generic/platform=iOS -allowProvisioningUpdates
    xcrun devicectl device install app --device "$IOS_DEVICE" target/ios/Build/Products/Debug-iphoneos/Rix.app
    xcrun devicectl device process launch --device "$IOS_DEVICE" top.cosas.rix

# The crate licence report ([052]), shipped in assets/licenses/. Needs `cargo install
# cargo-about --locked --features cli`. Run it after a dependency change.
licences:
    cargo about generate -c tools/about/about.toml -m mobile/Cargo.toml tools/about/about.hbs -o assets/licenses/third-party.md

# The site ([102]) to production on Vercel; `npx vercel login` once first.
site-deploy:
    npx --yes vercel deploy --prod --yes --cwd site

# Store art ([051]): the screenshots from the game, the Play feature graphic and icon, in
# target/store-art.
store-art:
    cargo build --features dev_tools
    tools/store-art.py
