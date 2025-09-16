#!/bin/bash

# Script to build an optimized version of TimeSpan for smaller Homebrew package

set -e

echo "🔧 Building optimized TimeSpan for smaller package size..."

# Build with optimizations
echo "📦 Building release version with optimizations..."
cargo build --release

# Check the size
echo "📊 Checking binary size..."
ls -lh target/release/timespan

# Test the optimized build
echo "🧪 Testing optimized build..."
./target/release/timespan --version
./target/release/timespan --help

echo "✅ Optimized build complete!"
echo ""
echo "📋 Size comparison:"
echo "   Original: ~4.1MB (with 2GB+ dependencies)"
echo "   Optimized: $(ls -lh target/release/timespan | awk '{print $5}') (binary only)"
echo ""
echo "💡 To use optimized formula:"
echo "   brew install --build-from-source ./Formula/timespan-optimized.rb"

