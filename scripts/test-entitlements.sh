#!/bin/bash

# Test script to verify entitlements are properly applied to QuickSpace builds
# This script helps debug hotkey issues by checking code signing and entitlements

set -e

echo "🔍 QuickSpace Entitlements Test Script"
echo "======================================"

# Find the QuickSpace app bundle
APP_PATH=""

# Check common locations
LOCATIONS=(
    "./src-tauri/target/release/bundle/macos/QuickSpace.app"
    "./src-tauri/target/universal-apple-darwin/release/bundle/macos/QuickSpace.app"
    "./src-tauri/target/aarch64-apple-darwin/release/bundle/macos/QuickSpace.app"
    "./src-tauri/target/x86_64-apple-darwin/release/bundle/macos/QuickSpace.app"
    "/Applications/QuickSpace.app"
)

for location in "${LOCATIONS[@]}"; do
    if [ -d "$location" ]; then
        APP_PATH="$location"
        echo "✅ Found QuickSpace at: $APP_PATH"
        break
    fi
done

if [ -z "$APP_PATH" ]; then
    echo "❌ QuickSpace.app not found. Please build the app first:"
    echo "   npm run tauri:build"
    echo "   or"
    echo "   npm run tauri:build -- --target universal-apple-darwin"
    exit 1
fi

echo ""
echo "🔐 Code Signing Information"
echo "=========================="

# Check if the app is signed
if codesign -dv "$APP_PATH" 2>/dev/null; then
    echo "✅ App is code signed"
    
    # Show signing details
    echo ""
    echo "📋 Signing Details:"
    codesign -dv "$APP_PATH" 2>&1 | grep -E "(Identifier|Authority|TeamIdentifier|Sealed Resources)"
    
    # Check for hardened runtime
    if codesign -dv "$APP_PATH" 2>&1 | grep -q "runtime"; then
        echo "✅ Hardened runtime enabled"
    else
        echo "⚠️  Hardened runtime not detected"
    fi
else
    echo "⚠️  App is not code signed (this is OK for local development)"
fi

echo ""
echo "🎫 Entitlements Check"
echo "==================="

# Extract and display entitlements
ENTITLEMENTS=$(codesign -d --entitlements - "$APP_PATH" 2>/dev/null)

if [ -n "$ENTITLEMENTS" ]; then
    echo "✅ Entitlements found:"
    echo "$ENTITLEMENTS" | xmllint --format -
    
    # Check for specific required entitlements
    echo ""
    echo "🔍 Required Entitlements Status:"
    
    REQUIRED_ENTITLEMENTS=(
        "com.apple.security.automation.apple-events"
        "com.apple.security.device.audio-input"
        "com.apple.security.cs.allow-unsigned-executable-memory"
        "com.apple.security.cs.disable-library-validation"
        "com.apple.security.cs.allow-jit"
        "com.apple.security.device.input-monitoring"
        "com.apple.security.device.screen-recording"
        "com.apple.security.cs.disable-executable-page-protection"
        "com.apple.security.temporary-exception.mach-lookup.global-name"
    )
    
    for entitlement in "${REQUIRED_ENTITLEMENTS[@]}"; do
        if echo "$ENTITLEMENTS" | grep -q "$entitlement"; then
            echo "✅ $entitlement"
        else
            echo "❌ $entitlement (MISSING - this may cause hotkey issues)"
        fi
    done
else
    echo "❌ No entitlements found"
    echo "   This will likely cause hotkey issues in distributed builds"
    echo "   Make sure to build with entitlements:"
    echo "   npm run tauri:build -- --config '{\"bundle\":{\"macOS\":{\"entitlements\":\"entitlements.release.plist\"}}}'"
fi

echo ""
echo "🔧 System Permissions Check"
echo "=========================="

# Check if the app has accessibility permissions
echo "Checking system permissions for QuickSpace..."

# Get the app's bundle identifier
BUNDLE_ID=$(defaults read "$APP_PATH/Contents/Info.plist" CFBundleIdentifier 2>/dev/null || echo "unknown")
echo "Bundle ID: $BUNDLE_ID"

# Check accessibility permissions
if command -v tccutil >/dev/null 2>&1; then
    echo "Note: Use System Preferences > Security & Privacy > Privacy to grant permissions"
else
    echo "Note: Grant Accessibility and Input Monitoring permissions in System Preferences"
fi

echo ""
echo "🚀 Testing Recommendations"
echo "========================="

if [ -n "$ENTITLEMENTS" ]; then
    echo "✅ App appears properly configured for distribution"
    echo "   If hotkeys still don't work:"
    echo "   1. Ensure Accessibility permission is granted"
    echo "   2. Ensure Input Monitoring permission is granted"
    echo "   3. Try restarting the app after granting permissions"
else
    echo "⚠️  App may not work properly when distributed"
    echo "   Recommended actions:"
    echo "   1. Build with release entitlements"
    echo "   2. Test the build locally before distributing"
    echo "   3. Verify CI workflow uses entitlements.release.plist"
fi

echo ""
echo "📝 Build Commands for Testing"
echo "============================"
echo "Local development build:"
echo "  npm run tauri:build"
echo ""
echo "Production-like build with entitlements:"
echo "  npm run tauri:build -- --config '{\"bundle\":{\"macOS\":{\"entitlements\":\"entitlements.release.plist\"}}}'"
echo ""
echo "Universal binary with entitlements:"
echo "  npm run tauri:build -- --target universal-apple-darwin --config '{\"bundle\":{\"macOS\":{\"entitlements\":\"entitlements.release.plist\"}}}'"

echo ""
echo "✨ Test complete!"
