#!/bin/bash

# Pre-commit hook for TimeSpan
# This script runs tests before allowing commits

set -e

echo "🔍 Running pre-commit checks..."

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Not in a git repository"
    exit 1
fi

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust toolchain."
    exit 1
fi

# Run sensitive data detection first
echo "🔒 Running sensitive data detection..."
if ! ./scripts/sensitive-data-check.sh; then
    echo "❌ Sensitive data detection failed. Please fix issues before committing."
    exit 1
fi

echo "📋 Checking code formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

echo "🔍 Running clippy lints..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy found issues. Please fix them before committing."
    exit 1
fi

echo "🧪 Running tests..."
if ! cargo test; then
    echo "❌ Tests failed. Please fix failing tests before committing."
    exit 1
fi

echo "🔨 Building project..."
if ! cargo build; then
    echo "❌ Build failed. Please fix build errors before committing."
    exit 1
fi

echo "✅ All pre-commit checks passed!"
echo "🚀 Proceeding with commit..."
