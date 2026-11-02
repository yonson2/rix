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

android_sdk := env_var_or_default("ANDROID_HOME", home_directory() / "Library/Android/sdk")
android_jdk := env_var_or_default("JAVA_HOME", "/Applications/Android Studio.app/Contents/jbr/Contents/Home")
android_dir := "mobile/android"

# Android debug APK ([021]): the Rust library via cargo-ndk into jniLibs, then Gradle.
android:
    ANDROID_HOME='{{android_sdk}}' cargo ndk -t arm64-v8a --platform 31 -o {{android_dir}}/app/src/main/jniLibs build --package rix_mobile
    cd {{android_dir}} && ANDROID_HOME='{{android_sdk}}' JAVA_HOME='{{android_jdk}}' ./gradlew --quiet assembleDebug

# Android release APK, unsigned until [054].
android-release:
    ANDROID_HOME='{{android_sdk}}' cargo ndk -t arm64-v8a --platform 31 -o {{android_dir}}/app/src/main/jniLibs build --release --package rix_mobile
    cd {{android_dir}} && ANDROID_HOME='{{android_sdk}}' JAVA_HOME='{{android_jdk}}' ./gradlew --quiet assembleRelease

# Install the debug APK on the running emulator or device and start it.
android-run: android
    '{{android_sdk}}/platform-tools/adb' install -r {{android_dir}}/app/build/outputs/apk/debug/app-debug.apk
    '{{android_sdk}}/platform-tools/adb' shell am start -n top.cosas.rix/.MainActivity
