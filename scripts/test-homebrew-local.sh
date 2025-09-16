#!/bin/bash

# Script to test the Homebrew formula locally without requiring a tap

set -e

echo "🧪 Testing Homebrew formula locally for TimeSpan"

# Create a temporary tap directory
TEMP_TAP_DIR=$(mktemp -d)
TAP_NAME="hisgarden/timespan"
TAP_DIR="$TEMP_TAP_DIR/$TAP_NAME"

echo "📁 Creating temporary tap at: $TAP_DIR"
mkdir -p "$TAP_DIR"

# Copy the formula to the temporary tap
cp "Formula/timespan-local.rb" "$TAP_DIR/timespan.rb"

# Test the formula syntax
echo "🔍 Checking formula syntax..."
brew audit --strict --formula "$TAP_DIR/timespan.rb"

echo "✅ Formula syntax is valid!"

# Install from the temporary tap
echo "🔨 Installing timespan from temporary tap..."
brew install "$TAP_DIR/timespan.rb"

# Test the installation
echo "🧪 Testing installation..."
timespan --version
timespan --help

# Create a test database and run basic tests
TEST_DB="/tmp/timespan-test.db"
rm -f "$TEST_DB"

echo "📝 Testing basic functionality..."
timespan --database "$TEST_DB" project create "Test Project"
timespan --database "$TEST_DB" project list
timespan --database "$TEST_DB" status

echo "✅ Basic functionality test passed!"

# Clean up
echo "🧹 Cleaning up..."
brew uninstall timespan 2>/dev/null || true
rm -rf "$TEMP_TAP_DIR"
rm -f "$TEST_DB"

echo "🎉 Homebrew formula test completed successfully!"