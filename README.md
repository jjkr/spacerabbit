# QuickSpace

A simple Tauri app that runs in the background and listens for global hotkeys on macOS.

## Features

- Headless background operation
- Global hotkey support (Alt+L)
- System tray integration

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
cargo tauri dev
```

### Building

```bash
# Build for production
cargo tauri build
```

## Usage

1. Run the app - it will start in the background with no visible window
2. Look for the QuickSpace icon in your system tray
3. Press Alt+L (Option+L) anywhere on your Mac to trigger the hotkey
4. Check the console output to see "Alt+L pressed!" messages
5. Right-click the system tray icon to quit the app

## Replacing icons

Write a new icon to `src-tauri/icons/OriginalLogo.png` then run:

```bash
npm run tauri -- icon src-tauri/icons/OriginalLogo.png
```

## Notes

- The app requires macOS accessibility permissions to register global hotkeys
