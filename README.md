<p align="center">
  <img height="284" src="https://github.com/jjkr/quickspace/blob/main/src-tauri/icons/Square284x284Logo.png">
</p>

<h1 align="center">QuickSpace</h1>

<p align="center">Fast workspace navigation for macOS.</p>

---

# QuickSpace

A simple Tauri app that runs in the background and listens for global hotkeys on macOS.

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
```

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
