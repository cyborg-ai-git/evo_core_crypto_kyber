#!/bin/bash
#===================================================================================================
# CyborgAI
# CC BY-NC-ND 4.0 Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International
# github: https://github.com/cyborg-ai-git
#===================================================================================================
PACKAGE_NAME="$(basename "$(pwd)")"
DIRECTORY_BASE=$(dirname "$(realpath "$0")")
DIR_OUTPUT="./output"
APP_NAME="cyborgai_peer"
#===================================================================================================
#clear
#===================================================================================================
CURRENT_TIME=$(date +"%Y.%-m.%-d%H%M")
echo "🟢 $CURRENT_TIME RUN crossover $PACKAGE_NAME [$DIRECTORY_BASE]"
#===================================================================================================
CURRENT_DIRECTORY=$(pwd)
#===================================================================================================
cd "$DIRECTORY_BASE" || exit
cd ..
#===================================================================================================
mkdir -p "$DIR_OUTPUT"
rm -Rf "$DIR_OUTPUT"/"$APP_NAME"*
#===================================================================================================
export RUST_LOG=debug
#===================================================================================================
TARGET="x86_64-unknown-linux-gnu"
echo "build -target $TARGET"
#time cross build --target $TARGET --release
time cargo zigbuild --target $TARGET --release
cp "target/$TARGET/release/$APP_NAME" "$DIR_OUTPUT"/"$APP_NAME"_"$TARGET"
#--------------------------------------------------------------------------------------------------
TARGET="x86_64-unknown-linux-musl"
echo "build -target $TARGET"
#time cross build --target $TARGET --release
time cargo zigbuild --target "$TARGET" --release
cp "target/$TARGET/release/$APP_NAME" "$DIR_OUTPUT"/"$APP_NAME"_"$TARGET"
#--------------------------------------------------------------------------------------------------
echo "build -target x86_64-apple-darwin"
#time cargo zigbuild --target x86_64-apple-darwin --release
#time cargo zigbuild --target aarch64-apple-darwin --release

# Then combine (this requires running on macOS)
#lipo -create \
#  target/x86_64-apple-darwin/release/your-binary \
#  target/aarch64-apple-darwin/release/your-binary \
#  -output target/your-binary-universal
#===================================================================================================
cd "$CURRENT_DIRECTORY" || exit
#===================================================================================================