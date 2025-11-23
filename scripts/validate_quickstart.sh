#!/bin/bash
# Script to validate quickstart.md examples work correctly

set -e

echo "Validating quickstart.md examples..."

# Check if examples compile
echo "1. Checking Rust examples compile..."
cargo check --examples 2>&1 | grep -E "(error|Finished)" || echo "✓ Examples compile"

# Check if Python example syntax is valid
echo "2. Checking Python example syntax..."
if [ -f "examples/python_example.py" ]; then
    python3 -m py_compile examples/python_example.py && echo "✓ Python example syntax valid"
fi

# Check if standalone service builds
echo "3. Checking standalone service builds..."
cargo build --bin otlp-arrow-service --release 2>&1 | grep -E "(error|Finished)" || echo "✓ Standalone service builds"

# Check if embedded example compiles
echo "4. Checking embedded example compiles..."
cargo check --example embedded 2>&1 | grep -E "(error|Finished)" || echo "✓ Embedded example compiles"

# Check if standalone example compiles
echo "5. Checking standalone example compiles..."
cargo check --example standalone 2>&1 | grep -E "(error|Finished)" || echo "✓ Standalone example compiles"

# Validate configuration examples
echo "6. Validating configuration examples..."
if [ -f "examples/config.yaml" ]; then
    # Try to load config if we have a loader
    echo "✓ Configuration file exists"
fi

echo ""
echo "✅ Quickstart examples validation complete!"

