#!/bin/bash

echo "🔨 Building Linux executable using WSL2..."

# Navigate to project directory
cd "$(dirname "$0")"

echo "📁 Current directory: $(pwd)"
echo "📋 Files in directory:"
ls -la

echo "🚀 Building websocket-trader for Linux..."
cargo build --release --bin websocket-trader

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo "📦 Executable location: target/release/websocket-trader"
    
    # Copy to deploy directory
    echo "📋 Copying to deploy directory..."
    cp target/release/websocket-trader deploy/websocket-trader
    chmod +x deploy/websocket-trader
    
    echo "✅ Linux executable ready for deployment!"
    ls -la deploy/websocket-trader
else
    echo "❌ Build failed!"
    exit 1
fi
