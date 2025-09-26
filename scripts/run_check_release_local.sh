#!/bin/bash
#===================================================================================================
# CyborgAI
# CC BY-NC-ND 4.0 Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International
# github: https://github.com/cyborg-ai-git
#===================================================================================================
PACKAGE_NAME="$(basename "$(pwd)")"
DIRECTORY_BASE=$(dirname "$(realpath "$0")")
#===================================================================================================
clear
#===================================================================================================
CURRENT_TIME=$(date +"%Y.%-m.%-d%H%M")
echo "🟢 $CURRENT_TIME RUN crossover $PACKAGE_NAME [$DIRECTORY_BASE]"
#===================================================================================================
CURRENT_DIRECTORY=$(pwd)
#===================================================================================================
cd "$DIRECTORY_BASE" || exit
cd ..
#===================================================================================================
export RUST_LOG=debug
cargo audit
cargo fmt
cargo clippy

#echo "build -target x86_64-unknown-linux-gnu"
#time cross build --target x86_64-unknown-linux-musl --release
#time cargo zigbuild --target x86_64-unknown-linux-gnu --release

echo "build -target x86_64-unknown-linux-musl"
#time cross build --target x86_64-unknown-linux-musl --release
time cargo zigbuild --target x86_64-unknown-linux-musl --release

#echo "build -target x86_64-apple-darwin"
#time cargo zigbuild --target x86_64-apple-darwin --release
#time cargo zigbuild --target aarch64-apple-darwin --release
#===================================================================================================
cd "$CURRENT_DIRECTORY" || exit
#===================================================================================================