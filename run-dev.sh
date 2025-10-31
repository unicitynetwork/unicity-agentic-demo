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

# Check if LLM base URL is set
if [ -z "$LLM_BASE_URL" ]; then
    echo "ℹ️  Note: LLM_BASE_URL not set, using default (z.ai)"
    echo "   Set it with: export LLM_BASE_URL=\"https://api.openai.com/v1/chat/completions\""
    export LLM_BASE_URL="https://api.z.ai/api/coding/paas/v4/chat/completions"
fi

# Check if LLM model is set
if [ -z "$LLM_MODEL" ]; then
    echo "ℹ️  Note: LLM_MODEL not set, using default (GLM-4.6)"
    echo "   Set it with: export LLM_MODEL=\"gpt-4\""
    export LLM_MODEL="GLM-4.6"
fi

# Check if ANTHROPIC_API_KEY is set (if you're using that)
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "ℹ️  Note: ANTHROPIC_API_KEY not set (using API_KEY instead)"
fi

echo "🔧 LLM Configuration:"
echo "   API Endpoint: $LLM_BASE_URL"
echo "   Model: $LLM_MODEL"
echo "   API Key: ${API_KEY:0:8}..."

# Navigate to Tauri directory
cd desktop

# Use cargo tauri dev instead of cargo run
# This properly bundles the app with Info.plist for macOS permissions
# and automatically handles frontend startup via beforeDevCommand
echo "🏗️  Starting Tauri dev mode (this will auto-start the frontend)..."
cargo tauri dev

# Note: cargo tauri dev handles cleanup automatically when you Ctrl+C