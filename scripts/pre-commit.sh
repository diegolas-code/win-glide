#!/bin/sh

# Exit immediately if a command exits with a non-zero status
set -e

echo "=== Running pre-commit formatting and linting checks ==="

# 1. Format the codebase
echo "Running: cargo fmt --all"
cargo fmt --all

# 2. Re-stage any files modified by cargo fmt that were already staged
# We query modified files and check if they are in the index
git diff --name-only | grep '\.rs$' | while read -r file; do
    if git diff --name-only --cached | grep -q "^$file$"; then
        echo "Re-staging formatted file: $file"
        git add "$file"
    fi
done

# 3. Run cargo clippy
echo "Running: cargo clippy --all-targets --all-features -- -D warnings"
# Disable set -e temporarily to handle Clippy's exit status and output custom error
set +e
cargo clippy --all-targets --all-features -- -D warnings
clippy_status=$?
set -e

if [ $clippy_status -ne 0 ]; then
    echo "❌ Error: cargo clippy failed with warnings or errors."
    echo "Please fix the issues before committing, or commit with --no-verify if absolutely necessary."
    exit 1
fi

echo "✅ All checks passed! Proceeding with commit."
exit 0
