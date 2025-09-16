# GitHub Actions CI/CD

This directory contains GitHub Actions workflows for the TimeSpan project to ensure code quality and prevent bad commits from being pushed to the repository.

## Workflows

### 1. CI Pipeline (`.github/workflows/ci.yml`)

Comprehensive CI/CD pipeline that runs on:
- Push to `main` and `develop` branches
- Pull requests to `main` and `develop` branches

**Jobs:**
- **Test Suite**: Runs on multiple Rust versions (stable, beta, nightly)
  - Code formatting check (`cargo fmt`)
  - Linting (`cargo clippy`)
  - Unit and integration tests (`cargo test`)
  - Build verification (`cargo build --release`)
- **Security Audit**: Runs `cargo audit` for vulnerability scanning
- **Build Matrix**: Cross-platform builds (Linux, Windows, macOS)
- **Pre-push Gate**: Final validation that blocks pushes if any checks fail

### 2. Pre-Push Validation (`.github/workflows/pre-push.yml`)

Lightweight validation that runs on every push to `main` and `develop`:
- Code formatting
- Clippy lints
- Unit tests
- Integration tests
- Build verification
- Security audit

## Local Development

### Pre-commit Hook

Install the pre-commit hook to run checks locally before commits:

```bash
# Install the pre-commit hook
./scripts/setup-pre-commit.sh

# The hook will now run before every commit
git commit -m "your message"
```

**What the hook checks:**
- ✅ Code formatting (`cargo fmt`)
- ✅ Linting (`cargo clippy`)
- ✅ Tests (`cargo test`)
- ✅ Build (`cargo build`)

### Manual Testing

You can run the same checks manually:

```bash
# Run all pre-commit checks
./scripts/pre-commit.sh

# Or run individual checks
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
```

## Branch Protection

Configure branch protection rules in GitHub:

1. Go to Settings → Branches
2. Add protection rule for `main` branch
3. Enable:
   - Require pull request reviews
   - Require status checks to pass
   - Require branches to be up to date
   - Required status checks: `pre-push-checks`, `test`, `security`, `build`

See `.github/BRANCH_PROTECTION.md` for detailed configuration.

## Bypassing Checks

### Emergency Situations

```bash
# Bypass local pre-commit hook
git commit --no-verify -m "emergency fix"

# Bypass GitHub checks (requires admin access)
git push --force-with-lease origin main
```

### Pre-commit Framework

For advanced users, you can also use the pre-commit framework:

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run on all files
pre-commit run --all-files
```

## Files

- `ci.yml` - Main CI/CD pipeline
- `pre-push.yml` - Pre-push validation
- `BRANCH_PROTECTION.md` - Branch protection configuration guide
- `scripts/pre-commit.sh` - Local pre-commit script
- `scripts/setup-pre-commit.sh` - Hook installation script
- `.pre-commit-config.yaml` - Pre-commit framework configuration

## Benefits

✅ **Prevents bad commits** from reaching the repository  
✅ **Catches issues early** in the development process  
✅ **Ensures code quality** with automated formatting and linting  
✅ **Runs comprehensive tests** before any push  
✅ **Security scanning** with cargo audit  
✅ **Cross-platform compatibility** verification  
✅ **Consistent development experience** across team members  

## Troubleshooting

### Common Issues

1. **Formatting errors**: Run `cargo fmt` to fix
2. **Clippy warnings**: Fix the warnings or add `#[allow(clippy::warning_name)]`
3. **Test failures**: Fix failing tests before committing
4. **Build errors**: Resolve compilation issues

### Getting Help

- Check the GitHub Actions logs for detailed error messages
- Run `cargo check` to see compilation errors
- Use `cargo clippy` to see linting suggestions
- Review the pre-commit script output for specific issues
