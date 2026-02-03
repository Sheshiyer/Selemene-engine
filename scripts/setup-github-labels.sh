#!/bin/bash
# Setup GitHub labels for Selemene Engine repository
# Run: ./scripts/setup-github-labels.sh

set -e

echo "🏷️  Setting up GitHub labels for Selemene Engine..."

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed."
    echo "Install it from: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "❌ Not authenticated with GitHub CLI"
    echo "Run: gh auth login"
    exit 1
fi

echo "✓ GitHub CLI authenticated"

# Engine Labels
echo ""
echo "📦 Creating engine labels..."
gh label create "engine-human-design" --color "FF6B6B" --description "Human Design engine" --force || true
gh label create "engine-gene-keys" --color "4ECDC4" --description "Gene Keys engine" --force || true
gh label create "engine-vimshottari" --color "45B7D1" --description "Vimshottari Dasha engine" --force || true
gh label create "engine-panchanga" --color "F7B731" --description "Panchanga engine" --force || true
gh label create "engine-numerology" --color "5F27CD" --description "Numerology engine" --force || true
gh label create "engine-biorhythm" --color "00D2D3" --description "Biorhythm engine" --force || true
gh label create "engine-vedic-clock" --color "FF9FF3" --description "Vedic Clock engine" --force || true
gh label create "engine-biofield" --color "54A0FF" --description "Biofield engine" --force || true
gh label create "engine-face-reading" --color "48DBFB" --description "Face Reading engine" --force || true
gh label create "engine-integration" --color "1DD1A1" --description "Integration layer" --force || true
gh label create "engine-core" --color "576574" --description "Core engine functionality" --force || true

# Wave Labels
echo ""
echo "🌊 Creating wave labels..."
gh label create "wave1" --color "0652DD" --description "Wave 1: Foundation (Auth, Docker, CORS)" --force || true
gh label create "wave2" --color "1289A7" --description "Wave 2: HD, Gene Keys, Vimshottari" --force || true
gh label create "wave3" --color "006266" --description "Wave 3: Vedic Clock, Integration Layer" --force || true
gh label create "wave4" --color "C23616" --description "Wave 4: Specialized Engines" --force || true

# Type Labels
echo ""
echo "🏷️  Creating type labels..."
gh label create "type-feature" --color "00B894" --description "New feature" --force || true
gh label create "type-bug" --color "D63031" --description "Bug fix" --force || true
gh label create "type-enhancement" --color "FDCB6E" --description "Enhancement to existing feature" --force || true
gh label create "type-docs" --color "6C5CE7" --description "Documentation" --force || true
gh label create "type-test" --color "00CEC9" --description "Testing" --force || true
gh label create "type-refactor" --color "A29BFE" --description "Code refactoring" --force || true
gh label create "type-performance" --color "FD79A8" --description "Performance improvement" --force || true

# Priority Labels
echo ""
echo "⚡ Creating priority labels..."
gh label create "priority-critical" --color "FF0000" --description "Critical - Blocking" --force || true
gh label create "priority-high" --color "FF6600" --description "High priority" --force || true
gh label create "priority-medium" --color "FFAA00" --description "Medium priority" --force || true
gh label create "priority-low" --color "00AA00" --description "Low priority" --force || true

# Status Labels
echo ""
echo "📊 Creating status labels..."
gh label create "status-blocked" --color "B33771" --description "Blocked by dependency" --force || true
gh label create "status-needs-review" --color "FD79A8" --description "Needs code review" --force || true
gh label create "status-needs-testing" --color "FDCB6E" --description "Needs testing" --force || true
gh label create "status-needs-docs" --color "6C5CE7" --description "Needs documentation" --force || true
gh label create "status-in-progress" --color "00B894" --description "Work in progress" --force || true

# Special Labels
echo ""
echo "✨ Creating special labels..."
gh label create "new-engine" --color "F79F1F" --description "Proposal for new engine" --force || true
gh label create "breaking-change" --color "EA2027" --description "Breaking API change" --force || true
gh label create "good-first-issue" --color "7F8FA6" --description "Good for newcomers" --force || true
gh label create "help-wanted" --color "33D9B2" --description "Extra attention needed" --force || true
gh label create "architecture" --color "3B3B98" --description "Architecture decision" --force || true

echo ""
echo "✅ All labels created successfully!"
echo ""
echo "📋 Next steps:"
echo "   1. Create GitHub Project: gh project create 'Consciousness Engine Roadmap'"
echo "   2. Import tasks: ./scripts/import-tasks-to-github.sh"
echo "   3. Configure automation: GitHub → Settings → Actions"
