#!/bin/bash
#===================================================================================================
# CyborgAI
# CC BY-NC-ND 4.0 Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International
# github: https://github.com/cyborg-ai-git
#===================================================================================================
DIRECTORY_BASE=$(dirname "$(realpath "$0")")
#---------------------------------------------------------------------------------------------------
clear
#---------------------------------------------------------------------------------------------------
CURRENT_TIME=$(date +"%Y%-m%-d%H%M")
#---------------------------------------------------------------------------------------------------
CURRENT_DIRECTORY=$(pwd)
#---------------------------------------------------------------------------------------------------
cd "$DIRECTORY_BASE" || exit
cd ..
#---------------------------------------------------------------------------------------------------
PACKAGE_NAME="$(basename "$(pwd)")"
GITHUB_BASE="https://github.com/cyborg-ai-git/$PACKAGE_NAME"
echo "🟢 $CURRENT_TIME - CREATE PRIVATE REPOSITORY GITHUB WITH GIT FLOW $GITHUB_BASE"
#---------------------------------------------------------------------------------------------------
# Initialize git repository
git init .
git add .
git commit -am "$PACKAGE_NAME init $CURRENT_TIME"

# Create GitHub repository
#gh auth login
gh repo create "$PACKAGE_NAME" --private --description "$PACKAGE_NAME"

# Set up master branch and push initial commit
git branch -M master
git remote add origin "$GITHUB_BASE.git"
git push -u origin master

echo "🔵 Setting up Git Flow branches..."

# Create develop branch from master
git checkout -b develop
git push -u origin develop

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
#---------------------------------------------------------------------------------------------------
cd "$CURRENT_DIRECTORY" || exit
#===================================================================================================