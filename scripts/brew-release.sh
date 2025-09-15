#!/bin/bash

# Script to help with Homebrew release process for TimeSpan

set -e

VERSION=${1:-"v1.0.2"}
REPO_URL="https://github.com/hisgarden/TimeSpan"

echo "🚀 Preparing Homebrew release for TimeSpan $VERSION"

# Check if git repo exists and has remote
if ! git remote get-url origin &>/dev/null; then
    echo "❌ No git remote 'origin' found. Please set up your GitHub repository first:"
    echo "   git remote add origin $REPO_URL"
    echo "   git push -u origin main"
    echo "   git push origin --tags"
    exit 1
fi

# Build the release binary
echo "🔨 Building release binary..."
cargo build --release

# Create release archive (simulate what GitHub would create)
ARCHIVE_NAME="TimeSpan-$VERSION.tar.gz"
echo "📦 Creating archive: $ARCHIVE_NAME"

# Create a temporary directory structure like GitHub would
TEMP_DIR=$(mktemp -d)
PROJECT_DIR="$TEMP_DIR/TimeSpan-${VERSION#v}"
mkdir -p "$PROJECT_DIR"

# Copy all files except target, .git, and other build artifacts
rsync -av \
    --exclude='.git*' \
    --exclude='target/' \
    --exclude='*.tar.gz' \
    --exclude='.DS_Store' \
    . "$PROJECT_DIR/"

# Create the archive
cd "$TEMP_DIR"
tar -czf "$ARCHIVE_NAME" "TimeSpan-${VERSION#v}"
mv "$ARCHIVE_NAME" -

# Calculate SHA256
SHA256=$(shasum -a 256 "$ARCHIVE_NAME" | cut -d' ' -f1)

echo "✅ Archive created: $ARCHIVE_NAME"
echo "🔑 SHA256: $SHA256"

# Update the Homebrew formula
echo "📝 Updating Homebrew formula..."
sed -i.bak "s|url.*|url \"$REPO_URL/archive/refs/tags/$VERSION.tar.gz\"|" Formula/timespan.rb
sed -i.bak "s|sha256.*|sha256 \"$SHA256\"|" Formula/timespan.rb
rm Formula/timespan.rb.bak

echo ""
echo "🎉 Release preparation complete!"
echo ""
echo "Next steps:"
echo "1. Commit and push the updated formula:"
echo "   git add Formula/timespan.rb"
echo "   git commit -m \"Update Homebrew formula for $VERSION\""
echo "   git push origin main"
echo ""
echo "2. Create a GitHub release with tag $VERSION and upload $ARCHIVE_NAME"
echo ""
echo "3. Test the installation:"
echo "   brew install --build-from-source Formula/timespan.rb"
echo ""
echo "4. For public tap, users can install with:"
echo "   brew tap jwen/timespan"
echo "   brew install timespan"

# Clean up
rm -rf "$TEMP_DIR"