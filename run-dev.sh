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

# Start frontend in background
echo "📦 Starting frontend development server..."
cd src-ui
npm install
npm run dev &
FRONTEND_PID=$!

# Wait a bit for frontend to start
sleep 3

# Start Tauri application
echo "🏗️  Starting Tauri application..."
cd ../src-tauri
cargo run

# Clean up frontend process on exit
kill $FRONTEND_PID 2>/dev/null