#!/usr/bin/env node

const express = require('express');
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const app = express();
const PORT = process.env.PORT || 10000;

console.log(`
╔═══════════════════════════════════════════════════════════════════════════════════╗
║                                                                                   ║
║    🚀 FOREX WEBSOCKET TRADER - DIRECT EXECUTABLE DEPLOYMENT 🚀                  ║
║         Running Pre-built Executable on Render                                   ║
║                                                                                   ║
╚═══════════════════════════════════════════════════════════════════════════════════╝
`);

// Check if the executable exists
const executablePath = path.join(__dirname, 'websocket-trader');
const executableExists = fs.existsSync(executablePath);

console.log(`📁 Executable path: ${executablePath}`);
console.log(`✅ Executable exists: ${executableExists}`);

if (!executableExists) {
    console.error('❌ ERROR: websocket-trader executable not found!');
    console.log('📋 Available files:');
    fs.readdirSync(__dirname).forEach(file => {
        console.log(`   - ${file}`);
    });
    process.exit(1);
}

// Make executable if needed
try {
    fs.chmodSync(executablePath, '755');
    console.log('🔧 Made executable file executable');
} catch (err) {
    console.log('⚠️  Could not chmod executable:', err.message);
}

// Start the Rust executable
console.log('🚀 Starting Forex WebSocket Trader...');

const traderProcess = spawn(executablePath, [], {
    stdio: 'inherit',
    env: {
        ...process.env,
        PORT: PORT.toString(),
        RUST_LOG: 'info',
        TRADING_MODE: 'DEMO'
    }
});

traderProcess.on('error', (err) => {
    console.error('❌ Failed to start trader process:', err);
    process.exit(1);
});

traderProcess.on('exit', (code, signal) => {
    console.log(`⚠️  Trader process exited with code ${code} and signal ${signal}`);
    process.exit(code || 1);
});

// Fallback Express server (in case the Rust executable doesn't start)
app.get('/health', (req, res) => {
    res.json({ 
        status: 'ok', 
        message: 'Node.js wrapper is running',
        executable_found: executableExists,
        timestamp: new Date().toISOString()
    });
});

app.get('/status', (req, res) => {
    res.json({ 
        service: 'forex-websocket-trader',
        wrapper: 'node.js',
        executable: executableExists ? 'found' : 'missing',
        port: PORT,
        timestamp: new Date().toISOString()
    });
});

// Start fallback server on a different port
const fallbackPort = parseInt(PORT) + 1;
app.listen(fallbackPort, () => {
    console.log(`🔄 Fallback Node.js server running on port ${fallbackPort}`);
});

console.log(`🎯 Main service should be running on port ${PORT}`);
console.log(`🌐 WebSocket endpoint: ws://localhost:${PORT}/ws`);
console.log(`📡 HTTP endpoints: http://localhost:${PORT}/health`);

// Keep the process alive
process.on('SIGTERM', () => {
    console.log('📴 Received SIGTERM, shutting down gracefully');
    if (traderProcess) {
        traderProcess.kill('SIGTERM');
    }
    process.exit(0);
});

process.on('SIGINT', () => {
    console.log('📴 Received SIGINT, shutting down gracefully');
    if (traderProcess) {
        traderProcess.kill('SIGINT');
    }
    process.exit(0);
});
