# Homebrew Installation Guide for TimeSpan

This guide covers how to install TimeSpan using Homebrew and how to maintain the Homebrew formula.

## For Users

### Quick Installation

```bash
# Add the TimeSpan tap
brew tap hisgarden/timespan

# Install TimeSpan
brew install timespan
```

### Alternative Installation Methods

#### Install from GitHub Releases (fastest)
```bash
brew install hisgarden/timespan/timespan
```

#### Install Development Version (latest code)
```bash
brew install hisgarden/timespan/timespan@dev
```

#### Install Locally (for testing)
```bash
# Clone the repository
git clone https://github.com/hisgarden/TimeSpan.git
cd TimeSpan

# Install using the local formula
brew install --build-from-source ./Formula/timespan-local.rb
```

## For Developers/Maintainers

### Setting Up the Homebrew Tap

1. **Create the tap structure:**
   ```bash
   ./scripts/setup-homebrew-tap.sh
   ```

2. **Push to GitHub:**
   ```bash
   cd ../homebrew-timespan
   git remote add origin https://github.com/hisgarden/homebrew-timespan.git
   git push -u origin main
   ```

### Releasing a New Version

1. **Update version in `Cargo.toml`**

2. **Generate the correct SHA256:**
   ```bash
   ./scripts/generate-homebrew-sha.sh v1.1.0
   ```

3. **Test the formula locally:**
   ```bash
   brew audit --strict --formula ./Formula/timespan.rb
   brew install --build-from-source ./Formula/timespan.rb
   ```

4. **Create GitHub release:**
   - Tag the release: `git tag v1.1.0 && git push origin v1.1.0`
   - Create release on GitHub with the tag
   - The archive URL should match the formula

5. **Update the tap:**
   ```bash
   cp Formula/timespan.rb ../homebrew-timespan/timespan.rb
   cd ../homebrew-timespan
   git add timespan.rb
   git commit -m "Update timespan to v1.1.0"
   git push origin main
   ```

### Testing the Formula

```bash
# Test formula syntax
brew audit --strict --formula ./Formula/timespan.rb

# Test installation
./scripts/test-homebrew-local.sh

# Test specific formula
brew install --build-from-source ./Formula/timespan-local.rb
```

## Available Formulas

### `timespan.rb`
- **Purpose**: Main production formula
- **Source**: GitHub release archives
- **Use case**: Regular users
- **Installation**: `brew install hisgarden/timespan/timespan`

### `timespan-local.rb` 
- **Purpose**: Local development testing
- **Source**: Local directory
- **Use case**: Testing changes before release
- **Installation**: `brew install ./Formula/timespan-local.rb`

### `timespan@dev.rb`
- **Purpose**: Development version
- **Source**: Latest code from main branch
- **Use case**: Early access to new features
- **Installation**: `brew install hisgarden/timespan/timespan@dev`

### `timespan-binary.rb.example`
- **Purpose**: Template for binary releases
- **Source**: Pre-compiled binaries
- **Use case**: Faster installation (when binary releases are available)
- **Status**: Example/template file

## Troubleshooting

### Common Issues

1. **"Formula not found"**
   ```bash
   brew tap hisgarden/timespan
   brew update
   ```

2. **"Build failed"**
   - Ensure Rust is installed: `brew install rust`
   - Check dependencies: `brew deps timespan`

3. **"Permission denied"**
   - Check Homebrew permissions: `brew doctor`

4. **Testing locally**
   ```bash
   brew install --build-from-source ./Formula/timespan-local.rb
   ```

### Getting Help

- Check the [main README](README.md) for general TimeSpan usage
- Report Homebrew-specific issues in the [homebrew-timespan repository](https://github.com/hisgarden/homebrew-timespan)
- For TimeSpan application issues, use the [main repository](https://github.com/hisgarden/TimeSpan)

## Formula Maintenance Checklist

- [ ] Version matches `Cargo.toml`
- [ ] SHA256 hash is correct for the version
- [ ] Tests pass with `brew audit --strict`
- [ ] Local installation works
- [ ] Help text matches actual output
- [ ] All dependencies are listed
- [ ] License is specified
- [ ] Description is accurate

---

Generated for TimeSpan v1.1.0