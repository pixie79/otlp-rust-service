#!/bin/bash
# Create labels for GitHub issues
# Usage: ./scripts/create_labels.sh

set -e

REPO="pixie79/otlp-rust-service"

echo "Creating GitHub labels..."

# Create labels with colors
gh label create "security" --repo "$REPO" --description "Security-related issues" --color "d73a4a" 2>/dev/null || echo "Label 'security' already exists"
gh label create "performance" --repo "$REPO" --description "Performance improvements" --color "0e8a16" 2>/dev/null || echo "Label 'performance' already exists"
gh label create "bug" --repo "$REPO" --description "Something isn't working" --color "d73a4a" 2>/dev/null || echo "Label 'bug' already exists"
gh label create "enhancement" --repo "$REPO" --description "New feature or request" --color "a2eeef" 2>/dev/null || echo "Label 'enhancement' already exists"
gh label create "documentation" --repo "$REPO" --description "Documentation improvements" --color "0075ca" 2>/dev/null || echo "Label 'documentation' already exists"
gh label create "testing" --repo "$REPO" --description "Testing improvements" --color "0e8a16" 2>/dev/null || echo "Label 'testing' already exists"
gh label create "critical" --repo "$REPO" --description "Critical priority issue" --color "b60205" 2>/dev/null || echo "Label 'critical' already exists"
gh label create "breaking-change" --repo "$REPO" --description "Breaking change" --color "b60205" 2>/dev/null || echo "Label 'breaking-change' already exists"
gh label create "incomplete" --repo "$REPO" --description "Incomplete implementation" --color "fbca04" 2>/dev/null || echo "Label 'incomplete' already exists"
gh label create "todo" --repo "$REPO" --description "TODO item" --color "fbca04" 2>/dev/null || echo "Label 'todo' already exists"
gh label create "python" --repo "$REPO" --description "Python-related" --color "1d76db" 2>/dev/null || echo "Label 'python' already exists"
gh label create "memory-safety" --repo "$REPO" --description "Memory safety issue" --color "b60205" 2>/dev/null || echo "Label 'memory-safety' already exists"
gh label create "logic-error" --repo "$REPO" --description "Logic error" --color "d73a4a" 2>/dev/null || echo "Label 'logic-error' already exists"
gh label create "memory" --repo "$REPO" --description "Memory-related" --color "0e8a16" 2>/dev/null || echo "Label 'memory' already exists"
gh label create "refactoring" --repo "$REPO" --description "Code refactoring" --color "c5def5" 2>/dev/null || echo "Label 'refactoring' already exists"

echo ""
echo "Labels created successfully!"

