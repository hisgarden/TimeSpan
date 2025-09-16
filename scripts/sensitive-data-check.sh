#!/bin/bash

# Sensitive Data Detection Script for TimeSpan
# This script scans the repository for sensitive client information before commits

set -e

echo "🔍 Running sensitive data detection..."

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

# Get the list of files that are staged for commit
STAGED_FILES=$(git diff --cached --name-only)

if [ -z "$STAGED_FILES" ]; then
    echo "ℹ️  No files staged for commit"
    exit 0
fi

echo "📋 Checking staged files for sensitive data..."

# Check each staged file for sensitive data
SENSITIVE_DATA_FOUND=false

for file in $STAGED_FILES; do
    if [ -f "$file" ]; then
        echo "🔍 Scanning: $file"
        
        # Check for specific user directory patterns (excluding generic ones)
        if grep -E "/Users/[^/]+/" "$file" | grep -v -E "/Users/(user|me)/" > /dev/null 2>&1; then
            echo "❌ Specific user directory path detected in: $file (use generic /Users/user/ or /Users/me/ instead)"
            SENSITIVE_DATA_FOUND=true
        fi
        
        # Check for client directory patterns (excluding generic ones)
        if grep -E "Clients/[^/]+" "$file" | grep -v -E "Clients/(ClientA|ClientB|ClientC)" > /dev/null 2>&1; then
            echo "❌ Specific client directory path detected in: $file (use generic ClientA, ClientB, ClientC instead)"
            SENSITIVE_DATA_FOUND=true
        fi
        
        # Check for workspace patterns that might be specific
        if grep -E "workspace/Clients" "$file" | grep -v -E "/path/to/client/repositories" > /dev/null 2>&1; then
            echo "❌ Specific workspace path detected in: $file (use generic /path/to/client/repositories instead)"
            SENSITIVE_DATA_FOUND=true
        fi
    fi
done

if [ "$SENSITIVE_DATA_FOUND" = true ]; then
    echo ""
    echo "🚨 SENSITIVE DATA DETECTED IN STAGED FILES! 🚨"
    echo ""
    echo "The following types of sensitive data were found:"
    echo "  - Specific user directory paths (non-generic /Users/ paths)"
    echo "  - Specific client directory paths (non-generic Clients/ paths)"
    echo "  - Specific workspace paths (non-generic workspace references)"
    echo ""
    echo "Please remove or sanitize this information before committing."
    echo "Use generic placeholders like:"
    echo "  - ClientA, ClientB, ClientC (instead of real client names)"
    echo "  - /Users/user/workspace/Clients (instead of specific paths)"
    echo "  - /path/to/client/repositories (instead of actual paths)"
    echo ""
    echo "❌ Commit blocked due to sensitive data detection"
    exit 1
fi

echo "✅ No sensitive data detected in staged files"

# Run the comprehensive sensitive data test
echo "🧪 Running comprehensive sensitive data detection test..."
if ! cargo test sensitive_data_detection --quiet; then
    echo "❌ Comprehensive sensitive data test failed"
    echo "Please check the test output above for details"
    exit 1
fi

echo "✅ All sensitive data checks passed!"
echo "🚀 Proceeding with commit..."
