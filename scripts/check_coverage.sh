#!/bin/bash
# Script to check code coverage per file (85% minimum requirement)

set -e

echo "Checking code coverage per file (minimum 85% required)..."

# Install tarpaulin if not present
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin --locked
fi

# Run coverage and check per-file coverage
cargo tarpaulin \
    --workspace \
    --all-features \
    --timeout 120 \
    --out Stdout \
    --exclude-files 'tests/*' \
    --exclude-files 'examples/*' \
    --exclude-files 'benches/*' \
    --exclude-files 'src/bin/*' \
    --exclude-files 'src/python/*' \
    --exclude-files 'target/*' \
    --exclude-files '**/main.rs' \
    --exclude-files '**/lib.rs' | \
grep -E "^\s+[0-9]+\.[0-9]+%" | \
awk '{print $2, $1}' | \
while read coverage file; do
    # Remove percentage sign and compare
    coverage_num=$(echo $coverage | sed 's/%//')
    if (( $(echo "$coverage_num < 85" | bc -l) )); then
        echo "❌ ERROR: $file has coverage $coverage (below 85% requirement)"
        exit 1
    else
        echo "✓ $file: $coverage"
    fi
done

echo ""
echo "✅ All files meet the 85% coverage requirement!"

