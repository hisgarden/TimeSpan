# Branch Protection Rules

This document describes the recommended branch protection rules for the TimeSpan repository to ensure code quality and prevent bad commits from being pushed.

## Recommended GitHub Branch Protection Settings

### For `main` branch:

1. **Require a pull request before merging**
   - ✅ Required
   - ✅ Require approvals: 1
   - ✅ Dismiss stale PR approvals when new commits are pushed
   - ✅ Require review from code owners

2. **Require status checks to pass before merging**
   - ✅ Required
   - ✅ Require branches to be up to date before merging
   - ✅ Status checks to require:
     - `pre-push-checks` (from pre-push.yml)
     - `test` (from ci.yml)
     - `security` (from ci.yml)
     - `build` (from ci.yml)

3. **Require conversation resolution before merging**
   - ✅ Required

4. **Restrict pushes that create files**
   - ✅ Required

5. **Allow force pushes**
   - ❌ Not allowed

6. **Allow deletions**
   - ❌ Not allowed

### For `develop` branch:

1. **Require a pull request before merging**
   - ✅ Required
   - ✅ Require approvals: 1

2. **Require status checks to pass before merging**
   - ✅ Required
   - ✅ Require branches to be up to date before merging
   - ✅ Status checks to require:
     - `pre-push-checks`
     - `test`

3. **Allow force pushes**
   - ❌ Not allowed

## Setting up Branch Protection

1. Go to your repository on GitHub
2. Navigate to Settings → Branches
3. Click "Add rule" or "Add branch protection rule"
4. Configure the rules as described above
5. Save the rule

## Local Pre-commit Hook

To prevent bad commits locally, install the pre-commit hook:

```bash
# Install the pre-commit hook
./scripts/setup-pre-commit.sh

# The hook will now run before every commit
git commit -m "your message"
```

## Bypassing Checks (Emergency Only)

In emergency situations, you can bypass checks:

```bash
# Bypass local pre-commit hook
git commit --no-verify -m "emergency fix"

# Bypass GitHub checks (requires admin access)
git push --force-with-lease origin main
```

## Workflow Files

- `.github/workflows/ci.yml` - Comprehensive CI/CD pipeline
- `.github/workflows/pre-push.yml` - Pre-push validation gate
- `.pre-commit-config.yaml` - Pre-commit hook configuration
- `scripts/pre-commit.sh` - Local pre-commit script
- `scripts/setup-pre-commit.sh` - Hook installation script
