#!/bin/bash

# Test build script for QuickSpace
# This script helps you test the build process locally before pushing to GitHub

set -e

echo "🚀 Testing QuickSpace build process..."

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed"
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo is not installed"
    exit 1
fi

echo "✅ All prerequisites found"

# Install dependencies
echo "📦 Installing dependencies..."
npm ci

# Check Rust formatting
echo "🎨 Checking Rust code formatting..."
cd src-tauri
cargo fmt --all -- --check
echo "✅ Rust formatting is correct"

# Run Rust tests
echo "🧪 Running Rust tests..."
cargo test
echo "✅ All tests passed"

cd ..

# Build the app
echo "🔨 Building QuickSpace..."
echo "   This may take a few minutes..."

# Build for current architecture first (faster)
npm run tauri:build

echo "✅ Build completed successfully!"

# Check if we can build universal binary
echo "🌍 Testing universal binary build..."
if npm run tauri:build -- --target universal-apple-darwin; then
    echo "✅ Universal binary build successful!"
else
    echo "⚠️  Universal binary build failed (this is normal if you don't have both architectures)"
fi

echo ""
echo "🎉 All tests passed! Your app is ready for CI/CD."
echo ""
echo "Next steps:"
echo "1. Commit your changes: git add . && git commit -m 'Add CI/CD pipeline'"
echo "2. Push to GitHub: git push"
echo "3. Create a release tag: git tag v1.0.0 && git push origin v1.0.0"
echo ""
echo "The GitHub Actions workflow will automatically build and release your app!"
