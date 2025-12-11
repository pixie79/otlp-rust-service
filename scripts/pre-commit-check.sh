#!/bin/bash
# Pre-commit checks for release preparation

set -e

echo "Running pre-commit checks..."

# Check 1: Ensure code compiles
echo "✓ Checking compilation..."
export PYO3_PYTHON=/opt/homebrew/opt/python@3.12/bin/python3.12
cargo check --all-features --workspace || {
    echo "✗ Compilation failed"
    exit 1
}

# Check 2: Run tests
echo "✓ Running tests..."
cargo test --all-features --workspace --quiet || {
    echo "✗ Tests failed"
    exit 1
}

# Check 3: Run clippy
echo "✓ Running clippy..."
cargo clippy --all-targets --all-features -- -A non_local_definitions -D warnings || {
    echo "✗ Clippy found issues"
    exit 1
}

# Check 4: Format check
echo "✓ Checking formatting..."
cargo fmt --all --check || {
    echo "✗ Code not formatted"
    exit 1
}

# Check 5: Verify version consistency
echo "✓ Checking version consistency..."
CARGO_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
PYPROJECT_VERSION=$(grep '^version = ' pyproject.toml 2>/dev/null | head -1 | sed 's/version = "\(.*\)"/\1/' || echo "")
FIRST_CHANGELOG_LINE=$(grep '^## \[' CHANGELOG.md | head -1)
FIRST_CHANGELOG_VERSION=$(echo "$FIRST_CHANGELOG_LINE" | sed 's/## \[\(.*\)\].*/\1/')
SECOND_CHANGELOG_LINE=$(grep '^## \[' CHANGELOG.md | sed -n '2p')
SECOND_CHANGELOG_VERSION=$(echo "$SECOND_CHANGELOG_LINE" | sed 's/## \[\(.*\)\].*/\1/')

# Check pyproject.toml version matches Cargo.toml
if [ -n "$PYPROJECT_VERSION" ] && [ "$PYPROJECT_VERSION" != "$CARGO_VERSION" ]; then
    echo "✗ Version mismatch: Cargo.toml has $CARGO_VERSION but pyproject.toml has $PYPROJECT_VERSION"
    exit 1
fi

# Check CHANGELOG version matches Cargo.toml
# If first entry is [Unreleased] or Unreleased, check the second entry
if [ "$FIRST_CHANGELOG_VERSION" = "[Unreleased]" ] || [ "$FIRST_CHANGELOG_VERSION" = "Unreleased" ]; then
    if [ -z "$SECOND_CHANGELOG_VERSION" ]; then
        echo "✗ CHANGELOG.md has [Unreleased] but no release version entry for $CARGO_VERSION"
        exit 1
    fi
    if [ "$SECOND_CHANGELOG_VERSION" != "$CARGO_VERSION" ]; then
        echo "✗ Version mismatch: Cargo.toml has $CARGO_VERSION but CHANGELOG.md release version is $SECOND_CHANGELOG_VERSION"
        exit 1
    fi
else
    # First entry is a version, it must match
    if [ "$FIRST_CHANGELOG_VERSION" != "$CARGO_VERSION" ]; then
        echo "✗ Version mismatch: Cargo.toml has $CARGO_VERSION but CHANGELOG.md has $FIRST_CHANGELOG_VERSION"
        exit 1
    fi
fi

if [ -n "$PYPROJECT_VERSION" ] && [ "$PYPROJECT_VERSION" != "$CARGO_VERSION" ]; then
    echo "✗ Version mismatch: Cargo.toml has $CARGO_VERSION but pyproject.toml has $PYPROJECT_VERSION"
    exit 1
fi

echo ""
echo "✓ All pre-commit checks passed!"

