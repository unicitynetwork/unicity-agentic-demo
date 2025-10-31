#!/bin/bash

# Build script for Unicity Agentic Demo UI

echo "🚀 Building Unicity Agentic Demo UI..."

# Check if API key is set
if [ -z "$API_KEY" ]; then
    echo "⚠️  Warning: API_KEY environment variable not set"
    echo "   Set it with: export API_KEY=\"your-api-key-here\""
fi

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