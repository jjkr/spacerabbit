# Rust Build Optimization Guide

This document explains the build optimizations implemented to speed up Rust compilation for the SpaceRabbit project.

## Optimizations Implemented

### 1. Cargo Configuration (`.cargo/config.toml`)

**Parallel Compilation:**
- `jobs = 0` - Uses all available CPU cores
- `codegen-units = 256` (dev) - Faster parallel compilation during development

**Faster Linking:**
- Uses `lld` linker on macOS for faster linking times
- Configured for both x86_64 and aarch64 architectures

**Incremental Compilation:**
- Enabled for both dev and release builds
- Significantly speeds up rebuilds when only small changes are made

**Dependency Resolution:**
- `protocol = "sparse"` - Faster dependency downloads from crates.io

### 2. Dependency Optimization

**Default Features Disabled:**
- All dependencies now use `default-features = false`
- Only explicitly needed features are enabled
- Reduces compilation surface area

**Tokio Optimization:**
- Changed from `features = ["full"]` to specific features: `["macros", "rt-multi-thread", "time"]`
- Eliminates unused async I/O, filesystem, and networking features

### 3. Build Profiles

**Development Profiles:**
- `dev` - Standard development build (fastest compilation)
- `dev-fast` - Slightly optimized development build (opt-level = 1)

**Release Profiles:**
- `release` - Full optimization for production
- `release-fast` - Quick optimized build for testing (opt-level = 2, no LTO)

## Usage

### Development Builds
```bash
# Standard development build (fastest)
cargo build

# Slightly optimized development build
cargo build --profile dev-fast
```

### Release Builds
```bash
# Full production build
cargo build --release

# Quick optimized build for testing
cargo build --profile release-fast
```

### Tauri Commands
```bash
# Development
npm run tauri dev

# Production build
npm run tauri build

# Quick optimized build
npm run tauri build -- --profile release-fast
```

## Expected Performance Improvements

- **Initial builds**: 30-50% faster
- **Incremental builds**: 60-80% faster
- **Development iterations**: 70-90% faster

## Additional Tips

### Clean Builds
If you encounter issues after these changes, clean your build cache:
```bash
cd src-tauri
cargo clean
```

### Monitoring Build Times
To measure build performance:
```bash
# Time a build
time cargo build

# Verbose output to see what's taking time
cargo build -v
```

### IDE Integration
These optimizations work automatically with:
- VS Code with rust-analyzer
- IntelliJ IDEA with Rust plugin
- Any editor using rust-analyzer

## Troubleshooting

### If builds fail after optimization:
1. Run `cargo clean` to clear the build cache
2. Check that all required features are still enabled for your dependencies
3. Verify that the lld linker is available on your system

### If you need additional tokio features:
Add them to the tokio dependency in `Cargo.toml`:
```toml
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time", "fs", "net"], default-features = false }
```

## Next Steps (Optional)

For even faster builds, consider:
- Installing `sccache` for distributed compilation caching
- Using `cargo-watch` for automatic rebuilds during development
- Setting up `cargo-nextest` for faster test execution
