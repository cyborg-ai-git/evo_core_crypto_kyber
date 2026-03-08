#!/bin/bash
#===================================================================================================
# CyborgAI
# CC BY-NC-ND 4.0 Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International
# github: https://github.com/cyborg-ai-git
#===================================================================================================
PACKAGE_NAME="$(basename "$(pwd)")"
DIRECTORY_BASE=$(dirname "$(realpath "$0")")
CURRENT_TIME=$(date +"%Y.%-m.%-d%H%M")
#===================================================================================================
clear
#---------------------------------------------------------------------------------------------------
echo "Usage: $0 'release_message'"
echo "  - Without 'release': commits to current branch"
echo "  - With 'release': creates git flow release using Cargo.toml version"
#---------------------------------------------------------------------------------------------------
echo "🟢 $CURRENT_TIME - RUN githu release  $DIRECTORY_BASE"
#---------------------------------------------------------------------------------------------------
CURRENT_DIRECTORY=$(pwd)
#---------------------------------------------------------------------------------------------------
cd "$DIRECTORY_BASE" || exit
cd ..
#---------------------------------------------------------------------------------------------------
bash ./scripts_release/run_build_release_local.sh
#---------------------------------------------------------------------------------------------------
pwd
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

# Set commit message (will be updated for releases after version is read)
if [ -z "$1" ]; then
    comment="commit $CURRENT_TIME"
else
    comment="$1"
fi

echo "💬 Initial commit message: $comment"
#---------------------------------------------------------------------------------------------------
# Configure git and fetch updates
git config http.postBuffer 524288000
git fetch --all

# Add and commit changes
#git add .
#git commit -am "$comment"
#---------------------------------------------------------------------------------------------------
# Check if this is a release
if [ "release" = "release" ]; then
    echo "🚀 Creating git flow release..."

    # Read version from Cargo.toml
    if [ -f "Cargo.toml" ]; then
        # Try workspace.package version first
        VERSION=$(grep -A 10 '\[workspace\.package\]' Cargo.toml | grep -E '^version = ' | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"')
        
        # If not found, try regular package version
        if [ -z "$VERSION" ]; then
            VERSION=$(grep -E '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/' | tr -d '"')
        fi

        if [ -z "$VERSION" ]; then
            echo "❌ Could not read version from Cargo.toml"
            exit 1
        fi

        echo "📦 Found version in Cargo.toml: $VERSION"
        RELEASE_VERSION="$VERSION"
        
        # Update commit message for release if not provided
        if [ -z "$1" ]; then
            comment="Release $RELEASE_VERSION"
            echo "💬 Updated commit message for release: $comment"
        fi
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

    # Add and commit any pending changes to develop
    git add .
    git commit -m "Prepare release $RELEASE_VERSION" || echo "No changes to commit"
    git push origin develop

    # Create a tag on develop branch
    echo "🏷️ Creating tag: $RELEASE_VERSION"
    if git tag -a "$RELEASE_VERSION" -m "Release $RELEASE_VERSION" 2>/dev/null; then
        echo "✅ Tag $RELEASE_VERSION created successfully"
        git push origin "$RELEASE_VERSION"
    else
        echo "⚠️ Tag $RELEASE_VERSION already exists, using existing tag"
        # Force push the tag if it exists locally but might be different
        git push origin "$RELEASE_VERSION" || echo "Tag already exists on remote"
    fi

    # Create GitHub release with binaries from ./output directory
    echo "🚀 Creating GitHub release with binaries..."
    
    # Check if output directory exists and has files
    if [ -d "./output" ] && [ "$(ls -A ./output 2>/dev/null)" ]; then
        echo "📦 Found binaries in ./output directory"
        
        # Create release with all binaries as attachments
        RELEASE_NOTES="Release $RELEASE_VERSION

## Built Artifacts
This release includes pre-built binaries for multiple platforms.

## Installation
Download the appropriate binary for your platform from the assets below.

Tag: $RELEASE_VERSION"

        # Create the release and attach all files from output directory
        if gh release create "$RELEASE_VERSION" \
            --title "Release $RELEASE_VERSION" \
            --notes "$RELEASE_NOTES" \
            --target develop \
            --prerelease \
            ./output/*; then
            
            echo "✅ GitHub release created successfully with binaries!"
            echo "🔗 Release URL: https://github.com/$(gh repo view --json owner,name --jq '.owner.login + "/" + .name')/releases/tag/$RELEASE_VERSION"
        else
            echo "❌ Failed to create GitHub release with binaries"
            echo "ℹ️ Attempting to create release without binaries..."
            
            # Fallback: create release without binaries
            if gh release create "$RELEASE_VERSION" \
                --title "Release $RELEASE_VERSION" \
                --notes "$RELEASE_NOTES" \
                --target develop \
                --prerelease; then
                
                echo "✅ GitHub release created (without binaries)"
                
                # Try to upload binaries separately
                echo "📦 Attempting to upload binaries separately..."
                for binary in ./output/*; do
                    if [ -f "$binary" ]; then
                        echo "⬆️ Uploading $(basename "$binary")..."
                        gh release upload "$RELEASE_VERSION" "$binary" || echo "⚠️ Failed to upload $(basename "$binary")"
                    fi
                done
            else
                echo "❌ Failed to create GitHub release"
                exit 1
            fi
        fi
    else
        echo "⚠️ No binaries found in ./output directory"
        echo "ℹ️ Creating release without binary attachments..."
        
        # Create release without binaries
        RELEASE_NOTES="Release $RELEASE_VERSION

Tag: $RELEASE_VERSION"

        if gh release create "$RELEASE_VERSION" \
            --title "Release $RELEASE_VERSION" \
            --notes "$RELEASE_NOTES" \
            --target develop \
            --prerelease; then
            
            echo "✅ GitHub release created successfully!"
            echo "🔗 Release URL: https://github.com/$(gh repo view --json owner,name --jq '.owner.login + "/" + .name')/releases/tag/$RELEASE_VERSION"
        else
            echo "❌ Failed to create GitHub release"
            exit 1
        fi
    fi

    # Check if GitHub CLI is available
    if ! command -v gh &> /dev/null; then
        echo "❌ GitHub CLI (gh) is not installed. Please install it first:"
        echo "   brew install gh"
        echo "   or visit: https://cli.github.com/"
        exit 1
    fi

    # Check if user is authenticated
    if ! gh auth status &> /dev/null; then
        echo "❌ Not authenticated with GitHub CLI. Please run:"
        echo "   gh auth login"
        exit 1
    fi

    # Create pull request from develop to master
    echo "🔄 Creating pull request from develop to master..."
    PR_TITLE="Release $RELEASE_VERSION"
    PR_BODY="This PR contains the release $RELEASE_VERSION.

## Changes
- Version: $VERSION
- Tag: $RELEASE_VERSION

## Release Process
1. Review and approve this PR
2. Merge to master
3. GitHub Actions will automatically build and create the release with binaries

## Built Artifacts
The following binaries will be automatically built and attached to the release:
- Linux (x86_64, ARM, MIPS variants)
- Windows (x86_64)
- macOS (x86_64, ARM64)

Release tag: \`$RELEASE_VERSION\`"

    # Check if PR already exists
    if gh pr view develop --json state 2>/dev/null | grep -q "OPEN"; then
        echo "⚠️ Pull request from develop to master already exists"
        PR_URL=$(gh pr view develop --json url --jq '.url' 2>/dev/null || echo "")
        if [ -n "$PR_URL" ]; then
            echo "🔗 Existing pull request: $PR_URL"
        fi
        echo "📋 You can update the existing PR or close it to create a new one"
        exit 0
    fi

    # Create the pull request
    if gh pr create \
        --title "$PR_TITLE" \
        --body "$PR_BODY" \
        --base master \
        --head develop; then
        
        echo "🟢 Pull request created successfully!"
        echo "📋 Next steps:"
        echo "   1. Review the pull request"
        echo "   2. Approve and merge to master"
        echo "   3. GitHub Actions will build and create the release automatically"
        echo ""
        
        # Try to get the PR URL
        PR_URL=$(gh pr view develop --json url --jq '.url' 2>/dev/null || echo "")
        if [ -n "$PR_URL" ]; then
            echo "🔗 View the pull request: $PR_URL"
        else
            echo "🔗 Check GitHub for the pull request"
        fi
    else
        echo "❌ Failed to create pull request. Please check:"
        echo "   - GitHub CLI authentication"
        echo "   - Repository permissions"
        echo "   - Network connectivity"
        exit 1
    fi

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