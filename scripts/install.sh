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
echo "🟢 $CURRENT_TIME INSTALL $PACKAGE_NAME [$DIRECTORY_BASE]"
#===================================================================================================
CURRENT_DIRECTORY=$(pwd)
#===================================================================================================
cd "$DIRECTORY_BASE" || exit
cd ..
#===================================================================================================
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.bashrc"
cargo -V
cargo install audit
#cargo install cross
cargo install cargo-zigbuild
#===================================================================================================
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup target add x86_64-unknown-linux-musl
#===================================================================================================
# Create develop branch from master
git checkout -b develop
git pull
# Initialize git flow (this will set master as production and develop as development branch)
# Note: git flow init will prompt for branch names, but we can provide defaults
echo -e "master\ndevelop\nfeature/\nrelease/\nhotfix/\nsupport/\nv" | git flow init

echo "🟢 Git Flow initialized successfully!"
echo "📋 Branch structure:"
echo "   - master: production-ready code"
echo "   - develop: integration branch for features"
echo "   - feature/*: feature branches"
echo "   - release/*: release preparation branches"

# Switch back to develop branch (git flow default working branch)
git checkout develop
echo "🔵 Current branch: $(git branch --show-current)"
echo "🟢 Repository setup complete with Git Flow!"
#===================================================================================================
cd "$CURRENT_DIRECTORY" || exit
#===================================================================================================