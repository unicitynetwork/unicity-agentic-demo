#!/bin/bash

# Build script for Unicity Agentic Demo UI

echo "🚀 Building Unicity Agentic Demo UI..."

# Check if API key is set
if [ -z "$API_KEY" ]; then
    echo "⚠️  Warning: API_KEY environment variable not set"
    echo "   Set it with: export API_KEY=\"your-api-key-here\""
fi

# Check if LLM base URL is set
if [ -z "$LLM_BASE_URL" ]; then
    echo "ℹ️  Note: LLM_BASE_URL not set, using default (z.ai)"
    echo "   Set it with: export LLM_BASE_URL=\"https://api.openai.com/v1/chat/completions\""
fi

# Check if LLM model is set
if [ -z "$LLM_MODEL" ]; then
    echo "ℹ️  Note: LLM_MODEL not set, using default (GLM-4.6)"
    echo "   Set it with: export LLM_MODEL=\"gpt-4\""
fi

echo "🔧 LLM Configuration:"
echo "   API Endpoint: ${LLM_BASE_URL:-"https://api.z.ai/api/coding/paas/v4/chat/completions"}"
echo "   Model: ${LLM_MODEL:-"GLM-4.6"}"
echo "   API Key: ${API_KEY:0:8}..."

# Build frontend
echo "📦 Building frontend..."
cd frontend
npm install
npm run build

if [ $? -ne 0 ]; then
    echo "❌ Frontend build failed"
    exit 1
fi

echo "✅ Frontend built successfully"

# Go back to root directory
cd ..

# Build Tauri app
echo "🏗️  Building Tauri application..."
cd ../desktop
cargo tauri build

if [ $? -ne 0 ]; then
    echo "❌ Tauri build failed"
    exit 1
fi

echo "✅ Tauri application built successfully"
echo "🎉 Build complete! Check desktop/target/release/bundle/ for the macOS app"