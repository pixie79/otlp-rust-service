#!/bin/bash
# Script to migrate test files from Config { ... } to ConfigBuilder

set -e

# Find all test files
find tests -name "*.rs" -type f | while read -r file; do
    # Check if file uses Config { pattern
    if grep -q "Config {" "$file" && ! grep -q "ConfigBuilder" "$file"; then
        echo "Migrating: $file"
        
        # Add ConfigBuilder to imports if Config is imported
        if grep -q "use.*Config[^B]" "$file"; then
            sed -i '' 's/use otlp_arrow_library::{\(.*\)Config\([^B].*\)}/use otlp_arrow_library::{\1ConfigBuilder\2}/' "$file"
            sed -i '' 's/use otlp_arrow_library::Config;/use otlp_arrow_library::ConfigBuilder;/' "$file"
        fi
        
        echo "  Note: Manual migration needed for Config struct initializations"
    fi
done

echo "Migration script complete. Please review and manually update Config { ... } initializations."

