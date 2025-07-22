# CI/CD Setup Guide for QuickSpace

This guide explains how to set up GitHub Actions for building and distributing your QuickSpace Tauri app.

## Overview

We've set up two workflows:
- **Development Build** (`build-dev.yml`): Runs on every push/PR, creates unsigned builds for testing
- **Release Build** (`build-and-release.yml`): Runs on version tags, creates signed builds for distribution

## Setting Up Code Signing (Optional but Recommended)

Even for direct distribution, code signing helps users trust your app and reduces security warnings.

### 1. Get an Apple Developer Certificate

1. Join the [Apple Developer Program](https://developer.apple.com/programs/) ($99/year)
2. In Xcode or Developer Portal, create a "Developer ID Application" certificate
3. Export the certificate as a `.p12` file with a password

### 2. Configure GitHub Secrets

Go to your GitHub repository → Settings → Secrets and variables → Actions, and add:

| Secret Name | Description | Required |
|-------------|-------------|----------|
| `APPLE_CERTIFICATE` | Base64-encoded .p12 certificate file | Optional |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 file | Optional |
| `APPLE_SIGNING_IDENTITY` | Certificate name (e.g., "Developer ID Application: Your Name") | Optional |
| `APPLE_ID` | Your Apple ID email | Optional |
| `APPLE_PASSWORD` | App-specific password for your Apple ID | Optional |
| `APPLE_TEAM_ID` | Your Apple Developer Team ID | Optional |

### 3. Encode Your Certificate

```bash
# Convert your certificate to base64
base64 -i YourCertificate.p12 | pbcopy
# Paste the result into APPLE_CERTIFICATE secret
```

## Creating Releases

### Automatic Releases (Recommended)

1. Create and push a version tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. GitHub Actions will automatically:
   - Build universal binaries (Intel + Apple Silicon)
   - Sign the app (if certificates are configured)
   - Create a GitHub release with DMG files
   - Include installation instructions

### Manual Releases

You can also trigger builds manually:
1. Go to Actions tab in your GitHub repository
2. Select "Build and Release" workflow
3. Click "Run workflow"
4. Choose the branch and run

## Build Outputs

### Release Builds
The workflow creates three separate DMG files for maximum compatibility:

- `QuickSpace_v1.0.0_universal.dmg` - Universal binary (Intel + Apple Silicon) - **Recommended**
- `QuickSpace_v1.0.0_aarch64.dmg` - Apple Silicon only (M1/M2/M3) - Smaller file size
- `QuickSpace_v1.0.0_x64.dmg` - Intel only - For older Macs

**Note**: The universal binary is recommended for most users as it works on all Mac architectures.

### Development Builds
- Available as GitHub Actions artifacts
- Unsigned builds for testing
- Automatically deleted after 7 days

## Installation Instructions for Users

Since your app uses private APIs and isn't notarized, users need to:

1. **Download** the appropriate DMG for their Mac architecture
2. **Open** the DMG and drag QuickSpace to Applications
3. **Right-click** the app in Applications and select "Open"
4. **Click "Open"** when macOS shows the security warning
5. **Grant permissions** for Accessibility and Input Monitoring when prompted

## Troubleshooting

### Build Failures

**Rust compilation errors:**
- Check that all dependencies are compatible with the target architecture
- Ensure your code compiles locally with `cargo build --target universal-apple-darwin`

**Signing failures:**
- Verify your certificate is valid and not expired
- Check that the signing identity matches your certificate exactly
- Ensure the certificate password is correct

**Missing dependencies:**
- Make sure `package.json` and `Cargo.toml` are up to date
- Check that all required system frameworks are available

### Testing Locally

Before pushing tags, test your build locally:

```bash
# Install dependencies
npm ci

# Build for your current architecture
npm run tauri:build

# Build universal binary (requires both architectures)
npm run tauri:build -- --target universal-apple-darwin
```

## Entitlements and Permissions

QuickSpace requires specific macOS entitlements to function properly when distributed:

### Required Entitlements
- `com.apple.security.automation.apple-events` - For accessibility features and workspace switching
- `com.apple.security.device.audio-input` - For input monitoring and global hotkey detection
- `com.apple.security.cs.allow-unsigned-executable-memory` - For Core Graphics private APIs
- `com.apple.security.cs.disable-library-validation` - For native libraries like global-hotkey
- `com.apple.security.cs.allow-jit` - For Rust runtime optimizations
- `com.apple.security.cs.allow-dyld-environment-variables` - Required for hardened runtime

### Entitlements Files
- `entitlements.plist` - Development entitlements (more permissive)
- `entitlements.release.plist` - Production entitlements (minimal required permissions)

The CI workflow automatically uses the release entitlements for distribution builds.

## Security Considerations

- **Never commit certificates or passwords** to your repository
- **Use GitHub Secrets** for all sensitive information
- **Rotate app-specific passwords** regularly
- **Monitor your repository** for unauthorized access
- **Entitlements are embedded** in the signed binary and cannot be modified after signing

## Troubleshooting Hotkey Issues

If hotkeys don't work in distributed builds:

1. **Check entitlements**: Verify the app was signed with proper entitlements
2. **Verify permissions**: Ensure Accessibility and Input Monitoring are granted
3. **Check signing**: Use `codesign -d --entitlements - /path/to/app` to verify entitlements
4. **Test locally**: Build with `--config '{"bundle":{"macOS":{"entitlements":"entitlements.release.plist"}}}'`

## Next Steps

1. **Set up code signing** (optional but recommended)
2. **Test the workflow** by creating a test tag
3. **Customize release notes** in the workflow file
4. **Add update mechanisms** if you want automatic updates later

For questions or issues, check the GitHub Actions logs or create an issue in the repository.
