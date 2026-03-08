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
echo "Usage: $0 [issue_number] ['pull_request_description']"
echo "Finish working on an issue: create pull request to develop and close issue"
echo "If no issue number provided, will try to extract from current branch name"
echo ""
#---------------------------------------------------------------------------------------------------
CURRENT_TIME=$(date +"%Y.%-m.%-d%H%M")
CURRENT_DIRECTORY=$(pwd)
#---------------------------------------------------------------------------------------------------
cd "$DIRECTORY_BASE" || exit
cd ..
#---------------------------------------------------------------------------------------------------
# Check if git repository exists
if [ ! -d .git ]; then
    echo "❌ No git repository found. Please initialize git repository first."
    exit 1
fi

# Check if git flow is initialized
if ! git config --get gitflow.branch.master >/dev/null 2>&1; then
    echo "❌ Git flow not initialized. Please run git flow init first."
    exit 1
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
    exit 1
fi

echo "✅ Repository found: $(echo "$REPO_INFO" | grep -o '"name":"[^"]*"' | cut -d'"' -f4)"
#---------------------------------------------------------------------------------------------------
# Get current branch and validate it's a feature branch
CURRENT_BRANCH=$(git branch --show-current)
echo "🔵 Current branch: $CURRENT_BRANCH"

# Check if we're on a feature branch
case "$CURRENT_BRANCH" in
    feature/*)
        echo "✅ On feature branch: $CURRENT_BRANCH"
        ;;
    *)
        echo "❌ Not on a feature branch. You must be on a feature/* branch to finish an issue."
        echo "💡 Switch to your feature branch first: git checkout feature/your-branch-name"
        exit 1
        ;;
esac

# Extract branch name without feature/ prefix
BRANCH_NAME="${CURRENT_BRANCH#feature/}"
#---------------------------------------------------------------------------------------------------
# Determine issue number
if [ -n "$1" ]; then
    # Issue number provided as parameter
    case "$1" in
        ''|*[!0-9]*)
            echo "❌ Error: Issue number must be a number"
            echo "Usage: $0 [issue_number] ['pull_request_description']"
            exit 1
            ;;
    esac
    ISSUE_NUMBER="$1"
    echo "📋 Using provided issue number: #$ISSUE_NUMBER"
else
    # Try to extract issue number from branch name (format: issue_NUMBER_description)
    case "$BRANCH_NAME" in
        issue_[0-9]*)
            ISSUE_NUMBER=$(echo "$BRANCH_NAME" | sed 's/issue_\([0-9]\+\).*/\1/')
            echo "📋 Extracted issue number from branch: #$ISSUE_NUMBER"
            ;;
        *)
            echo "❌ Cannot extract issue number from branch name: $BRANCH_NAME"
            echo "💡 Branch should start with 'issue_NUMBER_' or provide issue number as parameter"
            echo "Usage: $0 issue_number ['pull_request_description']"
            exit 1
            ;;
    esac
fi

PR_DESCRIPTION="$2"
#---------------------------------------------------------------------------------------------------
# Verify issue exists and get details
echo "📋 Checking issue #$ISSUE_NUMBER..."

ISSUE_INFO=$(gh issue view "$ISSUE_NUMBER" --json title,state,body 2>&1)
ISSUE_EXIT_CODE=$?

if [ $ISSUE_EXIT_CODE -ne 0 ]; then
    echo "❌ Issue #$ISSUE_NUMBER not found or cannot be accessed"
    echo "Error: $ISSUE_INFO"
    exit 1
fi

# Extract issue title and state
ISSUE_TITLE=$(echo "$ISSUE_INFO" | grep -o '"title":"[^"]*"' | cut -d'"' -f4)
ISSUE_STATE=$(echo "$ISSUE_INFO" | grep -o '"state":"[^"]*"' | cut -d'"' -f4)

echo "✅ Found issue #$ISSUE_NUMBER: $ISSUE_TITLE"
echo "📊 Current status: $ISSUE_STATE"

# Warn if issue is already closed
if [ "$ISSUE_STATE" = "CLOSED" ]; then
    echo "⚠️  Warning: Issue #$ISSUE_NUMBER is already closed"
    printf "Do you want to continue anyway? (y/n): "
    read -r REPLY
    case "$REPLY" in
        [Yy]|[Yy][Ee][Ss]) ;;
        *) echo "❌ Aborted"; exit 1 ;;
    esac
fi
#---------------------------------------------------------------------------------------------------
echo "🟢 $CURRENT_TIME - FINISHING ISSUE #$ISSUE_NUMBER: $ISSUE_TITLE"
#---------------------------------------------------------------------------------------------------
# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "📝 Uncommitted changes found. Committing them..."
    git add .
    git commit -m "feat: complete work on issue #$ISSUE_NUMBER - $ISSUE_TITLE"
    echo "✅ Changes committed"
fi

# Prepare for pull request by rebasing on latest develop
echo "� Prehparing branch for pull request..."

# Switch to develop and pull latest
echo "📥 Updating develop branch..."
git checkout develop
git pull origin develop

# Switch back to feature branch and rebase on develop
echo "🔄 Rebasing feature branch on develop..."
git switch "$CURRENT_BRANCH"
git rebase develop

# Push the rebased feature branch
echo "📤 Pushing rebased feature branch..."
git push origin "$CURRENT_BRANCH" --force-with-lease
#---------------------------------------------------------------------------------------------------
# Create pull request
echo "🔀 Creating pull request..."

# Set default PR description if none provided
if [ -z "$PR_DESCRIPTION" ]; then
    PR_DESCRIPTION="## Summary

This PR addresses issue #$ISSUE_NUMBER: $ISSUE_TITLE

## Changes Made
- [List your changes here]

## Testing
- [Describe how you tested your changes]

## Related Issue
Closes #$ISSUE_NUMBER"
fi

# Create PR targeting develop branch
PR_OUTPUT=$(gh pr create \
    --title "Fix #$ISSUE_NUMBER: $ISSUE_TITLE" \
    --body "$PR_DESCRIPTION" \
    --base develop \
    --head "$CURRENT_BRANCH" 2>&1)

PR_EXIT_CODE=$?

if [ $PR_EXIT_CODE -ne 0 ]; then
    echo "❌ Failed to create pull request"
    echo "Error: $PR_OUTPUT"
    echo ""
    echo "💡 This might happen if:"
    echo "   1. A PR already exists for this branch"
    echo "   2. No changes between feature branch and develop"
    echo "   3. Branch is not pushed to remote"
    exit 1
fi

# Extract PR number from output
PR_NUMBER=$(echo "$PR_OUTPUT" | grep -o '/pull/[0-9]\+' | grep -o '[0-9]\+')

echo "✅ Pull request created successfully!"
echo "🔗 PR URL: $PR_OUTPUT"

if [ -n "$PR_NUMBER" ]; then
    echo "� PoR #$PR_NUMBER: Fix #$ISSUE_NUMBER: $ISSUE_TITLE"
fi
#---------------------------------------------------------------------------------------------------
# Close the issue (it will be automatically closed when PR is merged if using "Closes #NUMBER" in PR description)
echo "� Crlosing issue #$ISSUE_NUMBER..."

CLOSE_OUTPUT=$(gh issue close "$ISSUE_NUMBER" --comment "Resolved by PR $PR_OUTPUT" 2>&1)
CLOSE_EXIT_CODE=$?

if [ $CLOSE_EXIT_CODE -eq 0 ]; then
    echo "✅ Issue #$ISSUE_NUMBER closed successfully"
else
    echo "⚠️  Could not close issue automatically: $CLOSE_OUTPUT"
    echo "💡 You can close it manually: gh issue close $ISSUE_NUMBER"
fi

# Show the created PR
echo "📋 Viewing created pull request..."
gh pr view "$PR_NUMBER"
#---------------------------------------------------------------------------------------------------
echo ""
echo "🎉 SUCCESS! Issue workflow completed:"
echo "   📋 Issue: #$ISSUE_NUMBER - $ISSUE_TITLE (CLOSED)"
echo "   🔀 Pull Request: $PR_OUTPUT"
if [ -n "$PR_NUMBER" ]; then
    echo "   📋 PR Number: #$PR_NUMBER"
fi
echo "   🌿 Feature branch: $CURRENT_BRANCH (rebased and ready for review)"
echo "   🔄 Current branch: $CURRENT_BRANCH"
echo ""
echo "💡 Next steps:"
echo "   1. Wait for PR approval and merge"
echo "   2. After PR is merged, the remote feature branch will be deleted"
echo "   3. You can then switch to develop and pul"
#---------------------------------------------------------------------------------------------------
cd "$CURRENT_DIRECTORY" || exit
#===================================================================================================