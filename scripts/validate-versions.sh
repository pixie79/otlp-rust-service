#!/bin/bash
# Pre-commit hook script to validate version consistency
# Ensures Cargo.toml, pyproject.toml, and CHANGELOG.md versions match

set -e

echo "🔍 Validating version consistency..."

# Extract version from Cargo.toml
CARGO_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
if [ -z "$CARGO_VERSION" ]; then
    echo "❌ ERROR: Could not extract version from Cargo.toml"
    exit 1
fi
echo "   Cargo.toml version: $CARGO_VERSION"

# Extract version from pyproject.toml (check both [project] and [project.metadata] sections)
PYPROJECT_VERSION=$(grep -E '^version = ' pyproject.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ -z "$PYPROJECT_VERSION" ]; then
    echo "❌ ERROR: Could not extract version from pyproject.toml"
    exit 1
fi
echo "   pyproject.toml version: $PYPROJECT_VERSION"

# Extract version from CHANGELOG.md (first non-Unreleased version)
CHANGELOG_VERSION=$(grep -E '^## \[[0-9]+\.[0-9]+\.[0-9]+\]' CHANGELOG.md | head -1 | sed 's/^## \[\(.*\)\].*/\1/')
if [ -z "$CHANGELOG_VERSION" ]; then
    echo "⚠️  WARNING: No version found in CHANGELOG.md (only [Unreleased] entries)"
    echo "   This is acceptable for feature PRs, but version must be set before release"
    CHANGELOG_VERSION="SKIP"
fi

# Validate Cargo.toml matches pyproject.toml
if [ "$CARGO_VERSION" != "$PYPROJECT_VERSION" ]; then
    echo ""
    echo "❌ ERROR: Version mismatch detected!"
    echo "   Cargo.toml version: $CARGO_VERSION"
    echo "   pyproject.toml version: $PYPROJECT_VERSION"
    echo ""
    echo "Please ensure versions match before committing."
    echo "The Python bindings version must match the Rust crate version."
    exit 1
fi

# Validate Cargo.toml matches CHANGELOG.md (if CHANGELOG has a version)
if [ "$CHANGELOG_VERSION" != "SKIP" ] && [ "$CARGO_VERSION" != "$CHANGELOG_VERSION" ]; then
    echo ""
    echo "❌ ERROR: Version mismatch detected!"
    echo "   Cargo.toml version: $CARGO_VERSION"
    echo "   CHANGELOG.md version: $CHANGELOG_VERSION"
    echo ""
    echo "Please ensure versions match before committing."
    exit 1
fi

echo "✅ All versions match: $CARGO_VERSION"
exit 0

