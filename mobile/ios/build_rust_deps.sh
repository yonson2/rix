#!/usr/bin/env bash
# Xcode's "Rust" build phase: builds the `rix` binary for each iOS arch and puts it where
# Xcode expects the app executable. Based on Bevy's examples/mobile ([020]).

set -euvx

PROFILE=debug
RELFLAG=
if [[ "$CONFIGURATION" != "Debug" ]]; then
    PROFILE=release
    RELFLAG=--release
fi

# Xcode puts its toolchain first in PATH, which breaks the link with
# `ld: library 'System' not found` (rust-lang/rust#80817): use the system `cc`.
export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"

ROOT="$SRCROOT/../.."
export CARGO_TARGET_DIR="$ROOT/target"

IS_SIMULATOR=0
if [ "${LLVM_TARGET_TRIPLE_SUFFIX-}" = "-simulator" ]; then
  IS_SIMULATOR=1
fi

EXECUTABLES=
for arch in $ARCHS; do
  case "$arch" in
    x86_64)
      if [ $IS_SIMULATOR -eq 0 ]; then
        echo "Building for x86_64, but not a simulator build." >&2
        exit 2
      fi
      export CFLAGS_x86_64_apple_ios="-target x86_64-apple-ios"
      TARGET=x86_64-apple-ios
      ;;
    arm64)
      if [ $IS_SIMULATOR -eq 0 ]; then
        TARGET=aarch64-apple-ios
      else
        TARGET=aarch64-apple-ios-sim
      fi
  esac

  cargo build $RELFLAG --manifest-path "$ROOT/Cargo.toml" --target $TARGET --bin rix
  EXECUTABLES="$EXECUTABLES $ROOT/target/$TARGET/$PROFILE/rix"
done

lipo -create -output "$TARGET_BUILD_DIR/$EXECUTABLE_PATH" $EXECUTABLES

# Xcode links nothing here, so it makes no dSYM and strips nothing: make the dSYM from the
# Rust object files, then strip the binary; the dSYM keeps the symbols ([105]).
if [ "${DEBUG_INFORMATION_FORMAT-}" = "dwarf-with-dsym" ]; then
  dsymutil "$TARGET_BUILD_DIR/$EXECUTABLE_PATH" -o "$DWARF_DSYM_FOLDER_PATH/$DWARF_DSYM_FILE_NAME"
  strip "$TARGET_BUILD_DIR/$EXECUTABLE_PATH"
fi
