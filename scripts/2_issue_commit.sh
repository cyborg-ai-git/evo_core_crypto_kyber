#!/bin/bash
#===================================================================================================
# CyborgAI
# CC BY-NC-ND 4.0 Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International
# github: https://github.com/cyborg-ai-git
#===================================================================================================
PACKAGE_NAME="$(basename "$(pwd)")"
DIRECTORY_BASE=$(dirname "$(realpath "$0")")
clear

echo "Usage: $0 'commit_message'"
echo "  - Without 'release': commits to current branch"

CURRENT_TIME=$(date +"%Y.%-m.%-d%H%M")
#---------------------------------------------------------------------------------------------------
echo "🟢 $CURRENT_TIME - RUN git flow $ $DIRECTORY_BASE"
#---------------------------------------------------------------------------------------------------
CURRENT_DIRECTORY=$(pwd)
#---------------------------------------------------------------------------------------------------
cd "$DIRECTORY_BASE" || exit
cd ..
#---------------------------------------------------------------------------------------------------
# Check if git repository exists
if [ -d .git ]; then
    echo "📁 Git repository found"
else
    echo "❌ No git repository found. Creating one..."
    sh ./run_create_github_repository.sh
fi

#---------------------------------------------------------------------------------------------------
# Set commit message
if [ -z "$1" ]; then
    comment="commit $CURRENT_TIME"
else
    comment="$1"
fi

echo "💬 Commit message: $comment"
#---------------------------------------------------------------------------------------------------
# Configure git and fetch updates
git config http.postBuffer 524288000
git fetch --all
git pull origin develop
git rebase develop
# Add and commit changes
git add .
git commit -am "$comment"
#---------------------------------------------------------------------------------------------------
# Regular commit workflow
CURRENT_BRANCH=$(git branch --show-current)
echo "🔵 Working on branch: $CURRENT_BRANCH"

# Pull and push current branch
git pull --rebase origin "$CURRENT_BRANCH"
git push #--force-with-lease origin "$CURRENT_BRANCH"

echo "🟢 Changes pushed to $CURRENT_BRANCH"
#---------------------------------------------------------------------------------------------------
cd "$CURRENT_DIRECTORY" || exit
#===================================================================================================