#!/bin/bash
#===================================================================================================
# CyborgAI
# CC BY-NC-ND 4.0 Creative Commons Attribution-NonCommercial-NoDerivatives 4.0 International
# github: https://github.com/cyborg-ai-git
#===================================================================================================
DIRECTORY_BASE=$(dirname "$(realpath "$0")")
PACKAGE_NAME="$(basename "$(pwd)")"
#---------------------------------------------------------------------------------------------------
clear
#---------------------------------------------------------------------------------------------------
echo "Usage: $0 issue_number"
echo "Start working on an existing GitHub issue by creating a git flow feature branch"
echo ""
#---------------------------------------------------------------------------------------------------
CURRENT_TIME=$(date +"%Y.%-m.%-d%H%M")
CURRENT_DIRECTORY=$(pwd)
#---------------------------------------------------------------------------------------------------
cd "$DIRECTORY_BASE" || exit
cd ..
#---------------------------------------------------------------------------------------------------
# Check if we have required parameters
if [ -z "$1" ]; then
    echo "❌ Error: Issue number is required"
    echo "Usage: $0 issue_number"
    echo ""
    echo "💡 To see available issues, run: gh issue list"
    exit 1
fi

# Validate issue number is numeric
if ! [[ "$1" =~ ^[0-9]+$ ]]; then
    echo "❌ Error: Issue number must be a number"
    echo "Usage: $0 issue_number"
    exit 1
fi

ISSUE_NUMBER="$1"

echo "🟢 $CURRENT_TIME - START WORKING ON ISSUE #$ISSUE_NUMBER"
#---------------------------------------------------------------------------------------------------
# Check if git repository exists
if [ ! -d .git ]; then
    echo "❌ No git repository found. Please initialize git repository first."
    exit 1
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
#---------------------------------------------------------------------------------------------------
# Make sure we're authenticated with GitHub
echo "🔐 Checking GitHub authentication..."
if ! gh auth status >/dev/null 2>&1; then
    echo "❌ Not authenticated with GitHub. Please run: gh auth login"
    exit 1
fi

# Check if we're in a GitHub repository
echo "🔍 Checking GitHub repository..."
REPO_INFO=$(gh repo view --json name,owner 2>&1)
if [ $? -ne 0 ]; then
    echo "❌ Not in a valid GitHub repository or repository not found on GitHub"
    echo "Debug info: $REPO_INFO"
    echo ""
    echo "💡 Make sure:"
    echo "   1. You're in the correct directory"
    echo "   2. The repository exists on GitHub"
    echo "   3. You have access to the repository"
    exit 1
fi

echo "✅ Repository found: $(echo "$REPO_INFO" | grep -o '"name":"[^"]*"' | cut -d'"' -f4)"
#---------------------------------------------------------------------------------------------------
# Check if issue exists and get details
echo "📋 Checking issue #$ISSUE_NUMBER..."

ISSUE_INFO=$(gh issue view "$ISSUE_NUMBER" --json title,state,body 2>&1)
ISSUE_EXIT_CODE=$?

if [ $ISSUE_EXIT_CODE -ne 0 ]; then
    echo "❌ Issue #$ISSUE_NUMBER not found or cannot be accessed"
    echo "Error: $ISSUE_INFO"
    echo ""
    echo "💡 Available issues:"
    gh issue list --limit 10
    exit 1
fi

# Extract issue title and state
ISSUE_TITLE=$(echo "$ISSUE_INFO" | grep -o '"title":"[^"]*"' | cut -d'"' -f4)
ISSUE_STATE=$(echo "$ISSUE_INFO" | grep -o '"state":"[^"]*"' | cut -d'"' -f4)

echo "✅ Found issue #$ISSUE_NUMBER: $ISSUE_TITLE"
echo "📊 Status: $ISSUE_STATE"

# Check if issue is already closed
if [ "$ISSUE_STATE" = "CLOSED" ]; then
    echo "⚠️  Warning: This issue is already closed"
    read -p "Do you want to continue anyway? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Aborted"
        exit 1
    fi
fi

# Display issue details
echo ""
echo "📝 Issue Details:"
echo "════════════════════════════════════════════════════════════════"
gh issue view "$ISSUE_NUMBER"
echo "════════════════════════════════════════════════════════════════"
echo ""
#---------------------------------------------------------------------------------------------------
# Create sanitized branch name
# Remove special characters and replace spaces with underscores
SANITIZED_TITLE=$(echo "$ISSUE_TITLE" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9 ]//g' | sed 's/ /_/g' | sed 's/__*/_/g' | sed 's/^_\|_$//g')

# Create branch name with issue prefix
BRANCH_NAME="issue_${ISSUE_NUMBER}_${SANITIZED_TITLE}"

# Truncate branch name if too long (Git has limits around 250 chars, but let's be safe)
if [ ${#BRANCH_NAME} -gt 60 ]; then
    SANITIZED_TITLE=$(echo "$SANITIZED_TITLE" | cut -c1-$((60 - ${#ISSUE_NUMBER} - 7)))  # 7 for "issue__"
    BRANCH_NAME="issue_${ISSUE_NUMBER}_${SANITIZED_TITLE}"
fi

echo "🌿 Branch name: $BRANCH_NAME"

# Check if branch already exists locally
if git show-ref --verify --quiet refs/heads/feature/$BRANCH_NAME; then
    echo "⚠️  Branch feature/$BRANCH_NAME already exists locally"
    read -p "Do you want to switch to it? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git checkout "feature/$BRANCH_NAME"
        echo "🔄 Switched to existing branch: feature/$BRANCH_NAME"
        cd "$CURRENT_DIRECTORY" || exit
        exit 0
    else
        echo "❌ Aborted"
        exit 1
    fi
fi

# Check if branch already exists remotely
if git ls-remote --exit-code --heads origin "feature/$BRANCH_NAME" >/dev/null 2>&1; then
    echo "⚠️  Branch feature/$BRANCH_NAME already exists on remote"
    printf "Do you want to checkout the remote branch? (y/n): "
    read -r REPLY
    case "$REPLY" in
        [Yy]|[Yy][Ee][Ss])
            git checkout -b "feature/$BRANCH_NAME" "origin/feature/$BRANCH_NAME"
            echo "🔄 Checked out remote branch: feature/$BRANCH_NAME"
            cd "$CURRENT_DIRECTORY" || exit
            exit 0
            ;;
        *)
            echo "❌ Aborted"
            exit 1
            ;;
    esac
fi
#---------------------------------------------------------------------------------------------------
# Switch to develop branch and update
echo "🔄 Switching to develop branch..."
git checkout develop
git pull origin develop

# Create git flow feature branch
echo "🚀 Creating git flow feature branch..."
git flow feature start "$BRANCH_NAME"

# Push the new branch to remote
echo "📤 Pushing branch to remote..."
git push -u origin "feature/$BRANCH_NAME"

echo ""
echo "🟢 SUCCESS! Ready to work on issue:"
echo "   📋 Issue: #$ISSUE_NUMBER - $ISSUE_TITLE"
echo "   🌿 Branch: feature/$BRANCH_NAME"
echo "   📊 Status: $ISSUE_STATE"
echo ""
echo "💡 Next steps:"
echo "   1. Work on your changes in the feature/$BRANCH_NAME branch"
echo "   2. When done, finish the feature: git flow feature finish $BRANCH_NAME"
echo "   3. Close the issue: gh issue close $ISSUE_NUMBER"
echo ""
echo "🔧 Useful commands:"
echo "   - Check current branch: git branch --show-current"
echo "   - View issue: gh issue view $ISSUE_NUMBER"
echo "   - List all issues: gh issue list"
echo "   - Update issue: gh issue edit $ISSUE_NUMBER"
echo "   - Add comment to issue: gh issue comment $ISSUE_NUMBER --body 'your comment'"
#---------------------------------------------------------------------------------------------------
cd "$CURRENT_DIRECTORY" || exit
#===================================================================================================