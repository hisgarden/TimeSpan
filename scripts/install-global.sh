#!/bin/bash

# Script to install TimeSpan globally on the system

set -e

echo "🚀 Installing TimeSpan globally..."

# Build the release binary
echo "🔨 Building release binary..."
cargo build --release

# Check if binary exists
if [ ! -f "target/release/timespan" ]; then
    echo "❌ Build failed - binary not found"
    exit 1
fi

# Install to system PATH
echo "📦 Installing to /usr/local/bin/..."
sudo cp target/release/timespan /usr/local/bin/timespan

# Make executable (should already be, but just in case)
sudo chmod +x /usr/local/bin/timespan

echo "✅ TimeSpan installed successfully!"
echo ""
echo "You can now use 'timespan' from anywhere:"
echo "  timespan --help"
echo "  timespan status"
echo "  timespan project list"
echo ""

# Test the installation
if command -v timespan >/dev/null 2>&1; then
    echo "🧪 Testing installation..."
    timespan --version 2>/dev/null || echo "TimeSpan installed and ready to use!"
else
    echo "⚠️  Warning: timespan command not found in PATH"
    echo "   You may need to restart your terminal or add /usr/local/bin to your PATH"
fi