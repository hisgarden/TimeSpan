#!/bin/bash

# Script to generate the correct SHA256 hash for Homebrew formula
# This simulates what GitHub would create for a release archive

set -e

VERSION=${1:-"v1.1.0"}
REPO_URL="https://github.com/hisgarden/TimeSpan"

echo "🔍 Generating SHA256 hash for TimeSpan $VERSION Homebrew formula"

# Create a temporary directory structure like GitHub would
TEMP_DIR=$(mktemp -d)
PROJECT_DIR="$TEMP_DIR/TimeSpan-${VERSION#v}"
mkdir -p "$PROJECT_DIR"

echo "📁 Creating temporary project structure..."

# Copy all files except target, .git, and other build artifacts
# This mirrors what GitHub does when creating a source archive
rsync -av \
    --exclude='.git*' \
    --exclude='target/' \
    --exclude='*.tar.gz' \
    --exclude='.DS_Store' \
    --exclude='node_modules/' \
    . "$PROJECT_DIR/"

# Create the archive exactly as GitHub would
ARCHIVE_NAME="TimeSpan-${VERSION#v}.tar.gz"
echo "📦 Creating archive: $ARCHIVE_NAME"

cd "$TEMP_DIR"
tar -czf "$ARCHIVE_NAME" "TimeSpan-${VERSION#v}"

# Calculate SHA256
SHA256=$(shasum -a 256 "$ARCHIVE_NAME" | cut -d' ' -f1)

echo ""
echo "✅ Generated SHA256 hash for $VERSION:"
echo "🔑 SHA256: $SHA256"
echo ""

# Update the formula with the correct hash
echo "📝 Updating Homebrew formula..."
cd "$OLDPWD"
cp "Formula/timespan.rb" "Formula/timespan.rb.bak"
sed "s/PLACEHOLDER_SHA256/$SHA256/" "Formula/timespan.rb" > "Formula/timespan.rb.tmp"
mv "Formula/timespan.rb.tmp" "Formula/timespan.rb"

echo "✅ Updated Formula/timespan.rb with correct SHA256"
echo ""
echo "🧪 You can now test the formula locally:"
echo "   brew install --build-from-source ./Formula/timespan.rb"
echo ""
echo "📋 Formula details:"
echo "   URL: $REPO_URL/archive/refs/tags/$VERSION.tar.gz"
echo "   SHA256: $SHA256"

# Clean up
rm -rf "$TEMP_DIR"
mv "$TEMP_DIR/$ARCHIVE_NAME" . 2>/dev/null || true

echo ""
echo "🎉 Homebrew formula ready for release!"