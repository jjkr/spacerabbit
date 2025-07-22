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
npm run tauri:build

# Build universal binary (Intel + Apple Silicon)
npm run tauri:build -- --target universal-apple-darwin
```

## CI/CD and Distribution

This project uses GitHub Actions for automated building and releasing:

- **Development builds**: Triggered on every push to main/develop branches
- **Release builds**: Triggered when you push a version tag (e.g., `v1.0.0`)

### Creating a Release

1. Update version in `package.json` and `src-tauri/Cargo.toml`
2. Create and push a version tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```
3. GitHub Actions will automatically build and create a release with DMG files

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

## Notes

- The app requires macOS accessibility permissions to register global hotkeys
