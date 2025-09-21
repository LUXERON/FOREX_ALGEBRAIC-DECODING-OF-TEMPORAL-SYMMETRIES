#!/bin/bash

# Start script for Render deployment
echo "🚀 Starting Embedded Forex Trading System on Render..."

# Set default port if not provided
export PORT=${PORT:-8080}
export API_PORT=${PORT}

# Make executable if needed
chmod +x ./embedded-trader

# Start the embedded trader with HTTP API
echo "🌐 Starting HTTP API server on port $PORT..."
./embedded-trader
