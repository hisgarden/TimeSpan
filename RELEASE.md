# Release Guide for TimeSpan Homebrew Package

This guide outlines the process for releasing TimeSpan and updating the Homebrew formula.

## Prerequisites

1. Ensure you have a GitHub repository set up for TimeSpan
2. Homebrew tap created (`jwen/homebrew-timespan`)
3. Local development environment with Rust and Homebrew

## Release Process

### 1. Prepare the Release

```bash
# Update version in Cargo.toml
vim Cargo.toml

# Update CHANGELOG.md with new features/fixes
vim CHANGELOG.md

# Test the application
cargo test
cargo build --release
./target/release/timespan --help
```

### 2. Create GitHub Release

```bash
# Tag the release
git tag v1.0.3
git push origin v1.0.3

# Create a GitHub release at:
# https://github.com/jwen/TimeSpan/releases/new
# - Use tag v1.0.3
# - Upload the release archive
```

### 3. Update Homebrew Formula

Use the release script to automatically calculate SHA256 and update the formula:

```bash
# Run the release script
./scripts/brew-release.sh v1.0.3

# This will:
# - Build the release binary
# - Create the archive
# - Calculate SHA256 hash
# - Update Formula/timespan.rb with correct URL and hash
```

### 4. Test the Formula

```bash
# Test locally first
brew audit jwen/timespan/timespan

# Test installation (if you have a test environment)
brew uninstall timespan 2>/dev/null || true
brew install jwen/timespan/timespan

# Test functionality
timespan --help
timespan project create "Test Project"
timespan project list
```

### 5. Commit and Push Updates

```bash
# Commit the updated formula
git add Formula/timespan.rb
git commit -m "Update Homebrew formula for v1.0.3"
git push origin main

# Update the tap repository
cd /opt/homebrew/Library/Taps/jwen/homebrew-timespan
git add Formula/timespan.rb
git commit -m "Update TimeSpan to v1.0.3"
git push origin main
```

## Formula Structure

The Homebrew formula (`Formula/timespan.rb`) contains:

- **Description**: Brief description of TimeSpan
- **Homepage**: GitHub repository URL
- **URL**: Download URL for the source archive
- **SHA256**: Checksum of the source archive
- **Dependencies**: Build dependencies (Rust)
- **Install method**: How to build and install
- **Test suite**: Verification that installation works

## Troubleshooting

### Formula Audit Fails

```bash
# Check what's wrong
brew audit jwen/timespan/timespan

# Common issues:
# - Empty SHA256 hash
# - Incorrect description format
# - Missing dependencies
# - Trailing whitespace
```

### Installation Fails

```bash
# Install with verbose output
brew install -v jwen/timespan/timespan

# Check build logs
brew gist-logs jwen/timespan/timespan
```

### SHA256 Mismatch

1. Download the release archive manually
2. Calculate SHA256: `shasum -a 256 TimeSpan-v1.0.3.tar.gz`
3. Update the formula with correct hash
4. Test again

## User Installation

Once released, users can install TimeSpan with:

```bash
# Add the tap (first time only)
brew tap jwen/timespan

# Install or update TimeSpan
brew install timespan
brew upgrade timespan  # for updates
```

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
- [Creating Homebrew Taps](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)