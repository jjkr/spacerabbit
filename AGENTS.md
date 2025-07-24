# Agent Guidelines for SpaceRabbit

## Build Commands
- **Build**: `npm run tauri build` (production) or `cargo build` (dev)
- **Dev**: `npm run tauri dev` or `cargo run` (from src-tauri/)
- **Clean**: `npm run clean` (removes node_modules, build, target)
- **Test**: `cargo test` (from src-tauri/ - no npm test scripts defined)

## Code Style - Rust
- Use `snake_case` for functions/variables, `PascalCase` for types/structs
- Prefer `Result<T, String>` for error handling with descriptive messages
- Use `log::` macros (debug, info, warn, error) for logging
- Import external crates at top, then std, then local modules
- Use `unsafe` blocks only when necessary with clear comments
- Prefer explicit types over inference in function signatures

## Code Style - JavaScript
- Minimal frontend (headless app) - keep JS simple
- Use console.log for any frontend debugging

## Project Structure
- Frontend: `src/` (minimal HTML/JS for Tauri)
- Backend: `src-tauri/src/` (main Rust application)
- Scripts: `scripts/` (build helpers, git hooks)
- Main entry: `src-tauri/src/main.rs`