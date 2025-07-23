#!/bin/bash
#
# Setup script for git hooks
# This script configures git to use the custom hooks in .githooks/
#

set -e

echo "🔧 Setting up git hooks..."

# Configure git to use the .githooks directory
git config core.hooksPath .githooks

echo "✅ Git hooks configured successfully!"
echo ""
echo "The following hooks are now active:"
echo "  - pre-commit: Checks version consistency before commits"
echo ""
echo "💡 To disable hooks temporarily, use: git commit --no-verify"
