#!/bin/bash

# Development script for Unicity Agentic Demo UI

echo "🚀 Starting Unicity Agentic Demo UI in development mode..."

# Check if API key is set
if [ -z "$API_KEY" ]; then
    echo "⚠️  Warning: API_KEY environment variable not set"
    echo "   Set it with: export API_KEY=\"your-api-key-here\""
    echo "   Using demo key for development..."
    export API_KEY="demo-key"
fi

# Check if ANTHROPIC_API_KEY is set (if you're using that)
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "ℹ️  Note: ANTHROPIC_API_KEY not set (using API_KEY instead)"
fi

# Navigate to Tauri directory
cd src-tauri

# Use cargo tauri dev instead of cargo run
# This properly bundles the app with Info.plist for macOS permissions
# and automatically handles frontend startup via beforeDevCommand
echo "🏗️  Starting Tauri dev mode (this will auto-start the frontend)..."
cargo tauri dev

# Note: cargo tauri dev handles cleanup automatically when you Ctrl+C