#!/bin/bash
# Import tasks from JSON plans to GitHub Issues
# Run: ./scripts/import-tasks-to-github.sh

set -e

echo "📥 Importing tasks from JSON plans to GitHub Issues..."

# Check dependencies
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) not found. Install: https://cli.github.com/"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "❌ jq not found. Install: brew install jq"
    exit 1
fi

TASK_DIR=".claude/task-management"
REPO="Sheshiyer/Selemene-engine"

# Function to create issue from task
create_issue() {
    local file=$1
    local wave=$2
    local task_id=$3
    local title=$4
    local body=$5
    local labels=$6
    
    echo "Creating issue: $title"
    
    gh issue create \
        --repo "$REPO" \
        --title "$title" \
        --body "$body" \
        --label "$labels" \
        || echo "⚠️  Failed to create: $title"
}

# Process each JSON plan file
for plan_file in "$TASK_DIR"/*.json; do
    if [ ! -f "$plan_file" ]; then
        continue
    fi
    
    filename=$(basename "$plan_file")
    echo ""
    echo "📄 Processing: $filename"
    
    # Extract wave/phase identifier
    wave=$(jq -r '.metadata.phase // "unknown"' "$plan_file")
    
    # Count tasks
    task_count=$(jq '.tasks | length' "$plan_file")
    echo "   Found $task_count tasks"
    
    # Process each task
    jq -c '.tasks[]' "$plan_file" | while read -r task; do
        task_id=$(echo "$task" | jq -r '.id')
        title=$(echo "$task" | jq -r '.title')
        description=$(echo "$task" | jq -r '.description')
        status=$(echo "$task" | jq -r '.status')
        priority=$(echo "$task" | jq -r '.priority // "medium"')
        
        # Skip completed tasks
        if [ "$status" = "completed" ]; then
            echo "   ⏭️  Skipping completed: $title"
            continue
        fi
        
        # Build labels
        labels="$wave,priority-$priority"
        
        # Add engine labels if mentioned in title/description
        if echo "$title $description" | grep -qi "human design"; then
            labels="$labels,engine-human-design"
        fi
        if echo "$title $description" | grep -qi "gene keys"; then
            labels="$labels,engine-gene-keys"
        fi
        if echo "$title $description" | grep -qi "vimshottari"; then
            labels="$labels,engine-vimshottari"
        fi
        if echo "$title $description" | grep -qi "vedic clock"; then
            labels="$labels,engine-vedic-clock"
        fi
        if echo "$title $description" | grep -qi "numerology"; then
            labels="$labels,engine-numerology"
        fi
        if echo "$title $description" | grep -qi "panchanga"; then
            labels="$labels,engine-panchanga"
        fi
        
        # Build issue body
        body=$(cat <<EOF
## Task ID: $task_id

### Description
$description

### Status
- Current: $status
- Priority: $priority

### Source
- Plan: \`$filename\`
- Wave: $wave

### Notes
This issue was automatically imported from the JSON task management system.
EOF
)
        
        # Create issue
        create_issue "$plan_file" "$wave" "$task_id" "$title" "$body" "$labels"
        
        # Rate limit (GitHub API has limits)
        sleep 1
    done
done

echo ""
echo "✅ Task import complete!"
echo ""
echo "📋 Next steps:"
echo "   1. Review created issues: gh issue list"
echo "   2. Add to project: gh project item-add <project-id> --owner Sheshiyer --url <issue-url>"
echo "   3. Assign issues: gh issue edit <number> --assignee @me"
