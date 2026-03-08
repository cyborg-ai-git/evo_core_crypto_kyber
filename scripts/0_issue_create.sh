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
echo "Usage: $0 <type> 'issue_title' 'issue_description'"
echo "Creates a GitHub issue using templates and corresponding git flow feature branch"
echo ""
echo "Types:"
echo "  issue       - Bug report (.github/ISSUE_TEMPLATE/bug_report.md)"
echo "  feature     - Feature request (.github/ISSUE_TEMPLATE/feature_request.md)"
echo "  doc         - Documentation (.github/ISSUE_TEMPLATE/documentation.md)"
echo "  performance - Performance issue (.github/ISSUE_TEMPLATE/performance.md)"
echo ""
#---------------------------------------------------------------------------------------------------
CURRENT_TIME=$(date +"%Y.%-m.%-d%H%M")
CURRENT_DIRECTORY=$(pwd)
#---------------------------------------------------------------------------------------------------
cd "$DIRECTORY_BASE" || exit
cd ..
#---------------------------------------------------------------------------------------------------
# Check if we have required parameters
if [ -z "$1" ] || [ -z "$2" ] || [ -z "$3" ]; then
    echo "❌ Error: Issue type, title, and description are required"
    echo "Usage: $0 <type> 'issue_title' 'issue_description'"
    echo ""
    echo "Valid types: doc, feature"
    exit 1
fi

ISSUE_TYPE="$1"
ISSUE_TITLE="$2"
ISSUE_DESCRIPTION="$3"

# Validate issue type and set template path
case "$ISSUE_TYPE" in
    "feature")
        TEMPLATE_PATH=".github/ISSUE_TEMPLATE/feature_request.md"
        ;;
    "doc")
        TEMPLATE_PATH=".github/ISSUE_TEMPLATE/documentation.md"
        ;;
    *)
        echo "❌ Error: Invalid issue type '$ISSUE_TYPE'"
        echo "Valid types: issue, feature, doc, performance"
        exit 1
        ;;
esac

# Check if template exists
if [ ! -f "$TEMPLATE_PATH" ]; then
    echo "❌ Error: Template not found: $TEMPLATE_PATH"
    exit 1
fi

echo "📋 Using template: $TEMPLATE_PATH"

echo "🟢 $CURRENT_TIME - CREATE $ISSUE_TYPE ISSUE AND BRANCH FOR: $ISSUE_TITLE"
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
# Create GitHub issue using template
echo "📝 Creating GitHub issue using template..."

# Check if gh supports template editing
GH_VERSION=$(gh --version | head -n1 | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+')
echo "🔍 GitHub CLI version: $GH_VERSION"

# Create temporary file for issue content
TEMP_ISSUE_FILE=$(mktemp)

# Copy template content to temp file and automatically insert the issue title
cp "$TEMPLATE_PATH" "$TEMP_ISSUE_FILE"

# Replace the title placeholder in the template with the actual issue title
# This handles the YAML front matter title field
sed -i.bak "s/title: '\[.*\].*'/title: '$ISSUE_TITLE'/" "$TEMP_ISSUE_FILE" 2>/dev/null || true
sed -i.bak "s/title: \[.*\].*/title: '$ISSUE_TITLE'/" "$TEMP_ISSUE_FILE" 2>/dev/null || true

# Add the issue description to the template
# Insert description after the YAML front matter (after the second ---)
awk -v desc="$ISSUE_DESCRIPTION" '
/^---$/ { count++ }
count == 2 && !inserted { 
    print $0
    print ""
    print "## Issue Description"
    print desc
    print ""
    inserted = 1
    next
}
{ print }
' "$TEMP_ISSUE_FILE" > "$TEMP_ISSUE_FILE.tmp" && mv "$TEMP_ISSUE_FILE.tmp" "$TEMP_ISSUE_FILE"

# Clean up backup file created by sed
rm -f "$TEMP_ISSUE_FILE.bak"

echo "📝 Opening editor to customize issue content..."
echo "💡 Template loaded from: $TEMPLATE_PATH"
echo "💡 Issue title automatically inserted: $ISSUE_TITLE"
echo "💡 Issue description automatically inserted: $ISSUE_DESCRIPTION"
echo "💡 Please review and customize the template as needed"

# Use default editor (respects EDITOR environment variable, falls back to vim/nano)
if [ -n "$EDITOR" ]; then
    "$EDITOR" "$TEMP_ISSUE_FILE"
elif command -v vim >/dev/null 2>&1; then
    vim "$TEMP_ISSUE_FILE"
elif command -v nano >/dev/null 2>&1; then
    nano "$TEMP_ISSUE_FILE"
else
    echo "❌ No suitable editor found. Please set EDITOR environment variable or install vim/nano"
    rm "$TEMP_ISSUE_FILE"
    exit 1
fi

# Create issue with edited content
ISSUE_BODY=$(cat "$TEMP_ISSUE_FILE")
ISSUE_OUTPUT=$(gh issue create --title "$ISSUE_TITLE" --body "$ISSUE_BODY")

# Clean up temp file
rm "$TEMP_ISSUE_FILE"

# Extract issue number from output
ISSUE_NUMBER=$(echo "$ISSUE_OUTPUT" | grep -o '/issues/[0-9]\+' | sed 's/#//')

if [ -z "$ISSUE_NUMBER" ]; then
    echo "❌ Failed to create GitHub issue"
    exit 1
fi

echo "✅ Created GitHub issue #$ISSUE_NUMBER: $ISSUE_TITLE"
echo "🔗 Issue URL: $ISSUE_OUTPUT"
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
echo "🟢 SUCCESS! $ISSUE_TYPE issue and branch created:"
echo "   📋 Issue: #$ISSUE_NUMBER - $ISSUE_TITLE"
echo "   📋 Type: $ISSUE_TYPE"
echo "   � Template:: $TEMPLATE_PATH"
echo "   🌿 Branch: feature/$BRANCH_NAME"
echo "   � Isstue URL: $ISSUE_OUTPUT"
echo ""
gh issue list
#gh issue view $PR_NUMBER
echo ""
echo "💡 Next steps:"
echo "   1. Work on your changes in the feature/$BRANCH_NAME branch"
echo "   2. To start to work issue : ./scripts/run_issue_start.sh $ISSUE_NUMBER"
echo "   3. When done, finish the feature: ./scripts/run_issue_finish.sh $ISSUE_NUMBER"
echo "   4. The issue will be closed automatically when PR is merged"
git status
#---------------------------------------------------------------------------------------------------
cd "$CURRENT_DIRECTORY" || exit
#===================================================================================================