#!/bin/bash
#===================================================================================================
# CyborgAI
# CC BY-NC-ND 4.0 Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International
# github: https://github.com/cyborg-ai-git
#===================================================================================================

DIRECTORY_BASE=$(dirname "$(realpath "$0")")
PACKAGE_NAME="$(basename "$DIRECTORY_BASE")"
#clear

echo "Usage: $0 'commit_message' [release]"
echo "  - Without 'release': commits to current branch"
echo "  - With 'release': creates git flow release using Cargo.toml version"

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

# Check if git flow is initialized
if ! git config --get gitflow.branch.master >/dev/null 2>&1; then
    echo "❌ Git flow not initialized. Initializing now..."

    # Check if develop branch exists
    if git show-ref --verify --quiet refs/heads/develop; then
        echo "🔵 Develop branch found"
    else
        echo "🔵 Creating develop branch..."
        git checkout -b develop 2>/dev/null || git checkout develop
        git push -u origin develop 2>/dev/null || true
    fi

    # Initialize git flow with defaults (no hotfix branch)
    echo -e "master\ndevelop\nfeature/\nrelease/\n\nsupport/\nv" | git flow init

    echo "🟢 Git flow initialized successfully!"
fi

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

# Add and commit changes
git add .
git commit -am "$comment"
#---------------------------------------------------------------------------------------------------
# Check if this is a release
if [ "$2" = "release" ]; then
    echo "🚀 Creating git flow release..."

    # Read version from Cargo.toml
    if [ -f "Cargo.toml" ]; then
        VERSION=$(grep -E '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"')
        if [ -z "$VERSION" ]; then
            # Try workspace.package version
            VERSION=$(grep -A 10 '\[workspace\.package\]' Cargo.toml | grep -E '^version = ' | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"')
        fi

        if [ -z "$VERSION" ]; then
            echo "❌ Could not read version from Cargo.toml"
            exit 1
        fi

        echo "📦 Found version in Cargo.toml: $VERSION"
        RELEASE_VERSION="v$VERSION"
    else
        echo "❌ Cargo.toml not found"
        exit 1
    fi

    # Check current branch
    CURRENT_BRANCH=$(git branch --show-current)
    echo "🔵 Current branch: $CURRENT_BRANCH"

    # If not on develop, switch to develop
    if [ "$CURRENT_BRANCH" != "develop" ]; then
        echo "🔄 Switching to develop branch..."
        git checkout develop
        git pull --rebase origin develop
    else
        git pull --rebase origin develop
    fi

    # Start git flow release
    echo "🎯 Starting git flow release: $RELEASE_VERSION"
    git flow release start "$RELEASE_VERSION"

    # Push release branch
    git push -u origin "release/$RELEASE_VERSION"

    # Finish the release (this will merge to master and develop)
    echo "✅ Finishing git flow release..."
    git flow release finish "$RELEASE_VERSION" -m "Release $RELEASE_VERSION"

    # Push master and develop branches
    git push origin master
    git push origin develop
    git push origin --tags

    # Create GitHub release
    echo "🎉 Creating GitHub release..."
    #gh auth login
    gh release create "$RELEASE_VERSION" --title "$RELEASE_VERSION Release" --notes "$PACKAGE_NAME release $RELEASE_VERSION"

    echo "🟢 Release $RELEASE_VERSION completed successfully!"

else
    # Regular commit workflow
    CURRENT_BRANCH=$(git branch --show-current)
    echo "🔵 Working on branch: $CURRENT_BRANCH"

    # Pull and push current branch
    git pull --rebase origin "$CURRENT_BRANCH"
    git push --force-with-lease origin "$CURRENT_BRANCH"

    echo "🟢 Changes pushed to $CURRENT_BRANCH"
fi
#---------------------------------------------------------------------------------------------------
cd "$CURRENT_DIRECTORY" || exit
#===================================================================================================