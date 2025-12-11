#!/bin/bash
# Pre-commit checks for release preparation

set -e

echo "Running pre-commit checks..."

# Setup Python environment FIRST (before cargo commands that might build Python bindings)
# Check for common venv directory names (.venv, .venv312, etc.)
VENV_DIR=""
if [ -d ".venv312" ]; then
    VENV_DIR=".venv312"
elif [ -d ".venv" ]; then
    VENV_DIR=".venv"
fi

# Activate venv and set PYO3_PYTHON if venv exists
if [ -n "$VENV_DIR" ]; then
    echo "✓ Activating virtual environment ($VENV_DIR)..."
    source "$VENV_DIR/bin/activate"
    export PYO3_PYTHON="$VIRTUAL_ENV/bin/python"
    echo "  Using Python: $PYO3_PYTHON"
    
    # Install Python dependencies
    echo "✓ Installing Python dependencies..."
    pip install --quiet --upgrade pip || {
        echo "✗ Failed to upgrade pip"
        deactivate 2>/dev/null || true
        exit 1
    }
    pip install --quiet maturin pytest opentelemetry-api opentelemetry-sdk || {
        echo "✗ Failed to install Python dependencies"
        deactivate 2>/dev/null || true
        exit 1
    }
    echo "  Python dependencies installed"
else
    # Fallback to system Python if venv not found
    echo "⚠ No venv found, using system Python (Python tests will be skipped)"
    # Try to find Python 3.12 in common locations
    if [ -f "/opt/homebrew/opt/python@3.12/bin/python3.12" ]; then
        export PYO3_PYTHON="/opt/homebrew/opt/python@3.12/bin/python3.12"
    elif command -v python3.12 >/dev/null 2>&1; then
        export PYO3_PYTHON=$(command -v python3.12)
    elif command -v python3 >/dev/null 2>&1; then
        export PYO3_PYTHON=$(command -v python3)
    fi
fi

# Install Rust dependencies
echo "✓ Installing Rust dependencies..."
cargo fetch --quiet || {
    echo "✗ Failed to fetch Rust dependencies"
    [ -n "$VENV_DIR" ] && deactivate 2>/dev/null || true
    exit 1
}
echo "  Rust dependencies installed"

# Check 1: Ensure code compiles (without python-extension feature to avoid Python linking issues)
echo "✓ Checking compilation..."
cargo check --workspace || {
    echo "✗ Compilation failed"
    [ -n "$VENV_DIR" ] && deactivate 2>/dev/null || true
    exit 1
}

# Check 2: Run Rust tests (without python-extension feature)
# Note: python-extension feature requires Python linking and should only be used with maturin
echo "✓ Running Rust tests..."
cargo test --workspace --quiet || {
    echo "✗ Rust tests failed"
    [ -n "$VENV_DIR" ] && deactivate 2>/dev/null || true
    exit 1
}

# Check 2b: Build Python binaries and run Python tests (if venv exists)
if [ -n "$VENV_DIR" ]; then
    if command -v maturin >/dev/null 2>&1; then
        echo "✓ Building Python package (using $VENV_DIR)..."
        # Build the Python package with python-extension feature enabled
        maturin develop --release || {
            echo "✗ Failed to build Python package"
            deactivate 2>/dev/null || true
            exit 1
        }
        
        echo "✓ Running Python tests..."
        # Run Python tests in the activated venv
        python -m pytest tests/python/ -v || {
            echo "✗ Python tests failed"
            deactivate 2>/dev/null || true
            exit 1
        }
        deactivate
    else
        echo "⚠ Skipping Python tests (maturin not available in venv after installation)"
        deactivate 2>/dev/null || true
    fi
else
    echo "⚠ Skipping Python tests (venv not found - checked .venv and .venv312)"
fi

# Check 3: Run clippy (without python-extension feature to avoid Python linking issues)
echo "✓ Running clippy..."
cargo clippy --all-targets -- -A non_local_definitions -D warnings || {
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

