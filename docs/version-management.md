# Version Management Guide

This document explains the automated version management system for SpaceRabbit, which ensures version consistency across all project files.

## Overview

SpaceRabbit uses **package.json as the single source of truth** for versioning. All other version files are automatically synchronized to match the version in package.json.

### Files Managed

The following files contain version information and are kept in sync:

- `package.json` (source of truth)
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

## Developer Workflow

### Bumping Versions Locally

Use the provided npm scripts to bump versions:

```bash
# Patch version (0.9.0 → 0.9.1)
npm run version:patch

# Minor version (0.9.0 → 0.10.0)
npm run version:minor

# Major version (0.9.0 → 1.0.0)
npm run version:major
```

These commands will:
1. Update the version in `package.json`
2. Automatically sync the version to all other files
3. Create a git commit with the version change
4. Create a git tag

### Manual Version Sync

If you need to manually sync versions (e.g., after editing package.json directly):

```bash
npm run version:sync
```

### Checking Version Consistency

To verify all files have matching versions:

```bash
npm run version:check
```

This will show the status of each file and highlight any mismatches.

## Release Process

### Automated Release Workflow

1. **Create and push a git tag** with the desired version:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **GitHub Actions automatically**:
   - Extracts the version from the tag (removes 'v' prefix)
   - Updates all version files to match the tag
   - Builds the application with the correct version
   - Creates a GitHub release with the built artifacts

### Manual Release Steps

If you prefer manual control:

1. **Bump the version locally**:
   ```bash
   npm run version:minor  # or patch/major
   ```

2. **Push the changes and tag**:
   ```bash
   git push origin main
   git push origin --tags
   ```

3. **The release workflow will trigger automatically** when the tag is pushed.

## CI/CD Integration

### Development Builds

The development workflow (`.github/workflows/build-dev.yml`) includes a version consistency check:

- Runs `npm run version:check` before building
- Fails the build if versions are out of sync
- Prevents deployment of inconsistent versions

### Release Builds

The release workflow (`.github/workflows/build-and-release.yml`) automatically:

- Extracts version from git tag
- Updates all version files
- Verifies consistency before building
- Ensures released binaries have the correct version

## Scripts Reference

### `scripts/sync-version.js`

Synchronizes version from package.json to all other files.

**Usage**: `npm run version:sync`

**What it does**:
- Reads version from `package.json`
- Updates `src-tauri/Cargo.toml`
- Updates `src-tauri/tauri.conf.json`
- Updates `package-lock.json` (if present)

### `scripts/check-version-sync.js`

Validates that all version files are synchronized.

**Usage**: `npm run version:check`

**Exit codes**:
- `0`: All versions are in sync
- `1`: Version mismatch detected

## Troubleshooting

### Version Mismatch Detected

If you see version mismatches:

1. **Run the sync command**:
   ```bash
   npm run version:sync
   ```

2. **Verify the fix**:
   ```bash
   npm run version:check
   ```

3. **Commit the changes**:
   ```bash
   git add .
   git commit -m "chore: sync version files"
   ```

### CI Build Failing on Version Check

If the CI build fails with version inconsistencies:

1. **Sync versions locally**:
   ```bash
   npm run version:sync
   ```

2. **Commit and push**:
   ```bash
   git add .
   git commit -m "fix: sync version files"
   git push
   ```

### Release Build Using Wrong Version

If a release build uses an incorrect version:

1. **Check the git tag format** (should be `v1.2.3`)
2. **Verify the tag was pushed**: `git ls-remote --tags origin`
3. **Re-run the release workflow** if needed

## Best Practices

### Version Naming

- Use semantic versioning (SemVer): `MAJOR.MINOR.PATCH`
- Tag format: `v1.2.3` (with 'v' prefix)
- Examples: `v0.9.0`, `v1.0.0`, `v1.2.3`

### Development Workflow

1. **Always use npm scripts** for version bumps
2. **Check version consistency** before committing
3. **Test locally** before creating release tags
4. **Use descriptive commit messages** for version changes

### Release Workflow

1. **Test thoroughly** before releasing
2. **Use patch versions** for bug fixes
3. **Use minor versions** for new features
4. **Use major versions** for breaking changes
5. **Update release notes** in GitHub releases

## Migration from Manual Versioning

If you're migrating from manual version management:

1. **Ensure all current versions match**:
   ```bash
   npm run version:check
   ```

2. **If mismatched, sync them**:
   ```bash
   npm run version:sync
   ```

3. **Commit the synchronized versions**:
   ```bash
   git add .
   git commit -m "chore: implement automated version management"
   ```

4. **Start using the new workflow** for future version changes

## Advanced Usage

### Custom Version Updates

For non-standard version updates, you can:

1. **Manually edit package.json**
2. **Run the sync script**:
   ```bash
   npm run version:sync
   ```

### Pre-release Versions

For pre-release versions (e.g., `1.0.0-beta.1`):

```bash
npm version prerelease --preid=beta
npm run version:sync
```

### Hotfix Workflow

For urgent hotfixes:

1. **Create hotfix branch from main**
2. **Bump patch version**:
   ```bash
   npm run version:patch
   ```
3. **Push and create tag**:
   ```bash
   git push origin hotfix-branch
   git push origin --tags
