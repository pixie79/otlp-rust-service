#!/usr/bin/env python3
"""
Script to migrate Config { ... } patterns to ConfigBuilder in test files.
"""

import re
import sys
from pathlib import Path

def migrate_file(file_path: Path) -> bool:
    """Migrate a single file from Config { ... } to ConfigBuilder."""
    try:
        content = file_path.read_text()
        original_content = content
        
        # Skip if already migrated
        if 'ConfigBuilder' in content and 'Config {' not in content:
            return False
        
        # Update imports
        content = re.sub(
            r'use otlp_arrow_library::\{Config([^B].*?)\}',
            r'use otlp_arrow_library::{ConfigBuilder\1}',
            content
        )
        content = re.sub(
            r'use otlp_arrow_library::Config;',
            r'use otlp_arrow_library::ConfigBuilder;',
            content
        )
        
        # Pattern to match Config { ... } blocks
        # This is a simplified pattern - may need manual review for complex cases
        config_pattern = re.compile(
            r'Config\s*\{\s*'
            r'output_dir:\s*PathBuf::from\(([^)]+)\),\s*'
            r'write_interval_secs:\s*(\d+),?\s*'
            r'(?:trace_cleanup_interval_secs:\s*\d+,?\s*)?'
            r'(?:metric_cleanup_interval_secs:\s*\d+,?\s*)?'
            r'(?:protocols:\s*Default::default\(\),?\s*)?'
            r'(?:forwarding:\s*(?:None|Some\([^)]+\)),?\s*)?'
            r'(?:dashboard:\s*Default::default\(\),?\s*)?'
            r'\}',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_config(match):
            output_dir = match.group(1).strip()
            write_interval = match.group(2).strip()
            return f'ConfigBuilder::new()\n        .output_dir({output_dir})\n        .write_interval_secs({write_interval})\n        .build()\n        .unwrap()'
        
        content = config_pattern.sub(replace_config, content)
        
        # Remove unused PathBuf imports if Config is the only reason for it
        if 'PathBuf' in content and 'Config {' not in content:
            # Check if PathBuf is still used elsewhere
            pathbuf_uses = len(re.findall(r'\bPathBuf\b', content))
            if pathbuf_uses <= 2:  # Just the import and maybe one other use
                content = re.sub(r'use std::path::PathBuf;\n?', '', content)
        
        if content != original_content:
            file_path.write_text(content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)
        return False

def main():
    """Main entry point."""
    test_dir = Path('tests')
    migrated = 0
    
    for test_file in test_dir.rglob('*.rs'):
        if migrate_file(test_file):
            print(f"Migrated: {test_file}")
            migrated += 1
    
    print(f"\nMigrated {migrated} files")
    print("Please review changes and test!")

if __name__ == '__main__':
    main()

