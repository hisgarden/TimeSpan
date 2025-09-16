#!/bin/bash

# Script to set up a proper Homebrew tap for TimeSpan

set -e

GITHUB_USER="hisgarden"
TAP_NAME="timespan"
REPO_NAME="homebrew-$TAP_NAME"

echo "🍺 Setting up Homebrew tap for TimeSpan"
echo "📍 Tap: $GITHUB_USER/$TAP_NAME"
echo "📦 Repository: $REPO_NAME"

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Not in a git repository. Please run this from the TimeSpan project root."
    exit 1
fi

# Create the tap directory structure
TAP_DIR="../$REPO_NAME"
echo "📁 Creating tap directory at: $TAP_DIR"

if [ -d "$TAP_DIR" ]; then
    echo "⚠️  Tap directory already exists. Updating..."
    cd "$TAP_DIR"
    git pull origin main 2>/dev/null || echo "No remote configured yet"
    cd -
else
    mkdir -p "$TAP_DIR"
    cd "$TAP_DIR"
    git init
    echo "# Homebrew Tap for TimeSpan" > README.md
    echo "" >> README.md
    echo "## Installation" >> README.md
    echo "" >> README.md
    echo '```bash' >> README.md
    echo "brew tap $GITHUB_USER/$TAP_NAME" >> README.md
    echo "brew install timespan" >> README.md
    echo '```' >> README.md
    echo "" >> README.md
    echo "## Development" >> README.md
    echo "" >> README.md
    echo "This tap contains the Homebrew formula for TimeSpan, a local time tracking application." >> README.md
    cd -
fi

# Copy the main formula to the tap
echo "📋 Copying formula to tap..."
cp "Formula/timespan.rb" "$TAP_DIR/timespan.rb"

# Create a development formula that builds from source
echo "🔧 Creating development formula..."
cat > "$TAP_DIR/timespan@dev.rb" << 'EOF'
class TimespanAT1 < Formula
  desc "A local time tracking application built with Rust (development version)"
  homepage "https://github.com/hisgarden/TimeSpan"
  url "https://github.com/hisgarden/TimeSpan.git", branch: "main"
  license "MIT"
  head "https://github.com/hisgarden/TimeSpan.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "A local time tracking application", shell_output("#{bin}/timespan --help")

    # Test basic functionality with temporary database
    system "#{bin}/timespan", "--database", "#{testpath}/test.db", "project", "create", "Test Project"
    assert_match "Test Project", shell_output("#{bin}/timespan --database #{testpath}/test.db project list")

    # Test status (should show no active timer)
    assert_match "No active timer", shell_output("#{bin}/timespan --database #{testpath}/test.db status")
  end
end
EOF

# Initialize git in tap directory if needed
cd "$TAP_DIR"
if [ ! -d ".git" ]; then
    git init
fi

# Add files and create initial commit
git add .
if ! git diff --cached --quiet; then
    git commit -m "Add TimeSpan Homebrew formula

- Add main timespan formula
- Add development timespan@dev formula  
- Include basic README with installation instructions"
fi

echo ""
echo "✅ Homebrew tap created successfully!"
echo ""
echo "📍 Tap location: $TAP_DIR"
echo ""
echo "Next steps:"
echo "1. Push the tap to GitHub:"
echo "   cd $TAP_DIR"
echo "   git remote add origin https://github.com/$GITHUB_USER/$REPO_NAME.git"
echo "   git push -u origin main"
echo ""
echo "2. Test the tap locally:"
echo "   brew tap $GITHUB_USER/$TAP_NAME $TAP_DIR"
echo "   brew install timespan"
echo ""
echo "3. For development builds:"
echo "   brew install timespan@dev"

cd -