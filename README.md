<p align="center">
  <img height="150" src="https://github.com/jjkr/quickspace/blob/main/src-tauri/icons/Square150x150Logo.png">
</p>

<h1 align="center">QuickSpace</h1>

<p align="center">Fast workspace navigation for macOS.</p>

## Install
Get the latest installer [HERE](https://github.com/jjkr/quickspace/releases/latest).

## Features

- Fast workspace switching with hotkeys
- Menu bar icon with current workspace number

## Building and Running

### Prerequisites

- Rust and Cargo installed
- Node.js and npm
- Tauri CLI: `cargo install tauri-cli`

### Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri:dev
```

### Building

```bash
# Build for production
npm run tauri build

# Build universal binary (Intel + Apple Silicon)
npm run tauri build -- --target universal-apple-darwin
```

## CI/CD and Distribution

This project uses GitHub Actions for automated building and releasing:

- **Development builds**: Triggered on every push to main/develop branches
- **Release builds**: Triggered when you push a version tag (e.g., `v1.0.0`)

### Creating a Release

QuickSpace uses automated version management with `package.json` as the single source of truth:

1. **Bump the version** using npm scripts:
   ```bash
   # Patch version (0.9.0 → 0.9.1)
   npm run version:patch
   
   # Minor version (0.9.0 → 0.10.0)  
   npm run version:minor
   
   # Major version (0.9.0 → 1.0.0)
   npm run version:major
   ```

2. **Push the changes and tag**:
   ```bash
   git push origin main
   git push origin --tags
   ```

3. **GitHub Actions automatically**:
   - Extracts version from the git tag
   - Updates all version files to match
   - Builds and creates a release with DMG files

**Alternative**: Create a tag manually and let CI sync the versions:
```bash
git tag v1.0.0
git push origin v1.0.0
```

For more details, see [docs/version-management.md](docs/version-management.md).

### Code Signing Setup

For signed releases, configure these GitHub Secrets (optional but recommended):
- `APPLE_CERTIFICATE` - Base64-encoded Developer ID certificate
- `APPLE_CERTIFICATE_PASSWORD` - Certificate password
- `APPLE_SIGNING_IDENTITY` - Certificate name

See [docs/ci-setup.md](docs/ci-setup.md) for detailed setup instructions.

## Usage

1. Run the app - it will start in the background with no visible window
2. Look for the QuickSpace icon in your system tray
3. Press Alt+L (Option+L) anywhere on your Mac to go to the next workspace instantly!

## Replacing icons

Write a new icon to `src-tauri/icons/OriginalLogo.png` then run:

```bash
npm run tauri -- icon src-tauri/icons/OriginalLogo.png
```

## Troubleshooting

### Hotkeys Don't Work in Distributed Builds

If hotkeys work when building locally but not when installing from GitHub releases, this is likely an entitlements issue. The app now includes proper entitlements for distribution:

- `entitlements.plist` - Development entitlements (more permissive)
- `entitlements.release.plist` - Production entitlements (minimal required permissions)

To test your build with production entitlements locally:

```bash
# Test with release entitlements
npm run tauri build -- --config '{"bundle":{"macOS":{"entitlements":"entitlements.release.plist"}}}'

# Verify entitlements are applied
./scripts/test-entitlements.sh
```

### Required Permissions

The app requires these macOS permissions:
- **Accessibility**: For global hotkeys and workspace switching
- **Input Monitoring**: For detecting keyboard shortcuts

Grant these in System Preferences > Security & Privacy > Privacy when prompted.

## Notes

- The app requires macOS accessibility permissions to register global hotkeys
- Entitlements are automatically applied in CI builds for proper distribution
