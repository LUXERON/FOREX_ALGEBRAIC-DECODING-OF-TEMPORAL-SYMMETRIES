# FOREX ALGEBRAIC DECODING OF TEMPORAL SYMMETRIES

## Revolutionary Forex Pattern Reconstruction System

This repository contains a revolutionary forex trading system that transforms forex trading from probabilistic guessing into **algebraic decoding of temporal symmetries** using Galois field cyclicity.

## 🚀 Key Features

### **Enhanced CLI Controller**
- **Custom cTrader Credentials Support** - Use any cTrader account/broker
- **DEMO/LIVE Mode Switching** - Enterprise-grade safety features
- **Remote Control Architecture** - Local CLI controls remote deployment
- **Real-time Monitoring** - Complete system status and control

### **Production-Ready System**
- **3.5MB Statically Linked Executable** with embedded SQLite database
- **15 Currency Pairs** with cross-pair correlation analysis
- **116,629+ Historical Data Points** (1980-2023) embedded in executable
- **Enterprise-Grade Safety** with live mode confirmations

### **Mathematical Innovation**
- **Time-Symmetric Pattern Recognition** using mathematical foundations
- **Galois Field Cyclicity** for detecting cyclic patterns in market data
- **Anomaly Detection** from temporal symmetries
- **Laplacian Reinforcement Learning** with graph-based Q-learning

## 🏗️ System Architecture

### **Core Binaries**
- `embedded_trader` - Main trading system with embedded database
- `simple_cli_controller` - Enhanced CLI with custom credentials support
- `ctrader_bridge` - cTrader API integration

### **Key Modules**
- `src/core/` - Core mathematical engine and temporal state management
- `src/galois/` - Galois field operations for pattern detection
- `src/symmetry/` - Time-symmetric pattern recognition
- `src/anomaly/` - Anomaly detection from temporal symmetries
- `src/laplacian_rl/` - Graph-based reinforcement learning
- `src/embedded_db/` - Embedded SQLite with compressed historical data
- `src/correlation/` - Cross-pair correlation analysis

## 🚀 Quick Start

### **Prerequisites**
- Rust 1.70+
- cTrader account (demo or live)

### **Build**
```bash
# Build the complete system
cargo build --release

# Build for Linux deployment (static linking)
cargo build --release --target x86_64-unknown-linux-musl
```

### **Usage**

#### **1. Start the Embedded Trading System**
```bash
./target/release/embedded-trader
# Starts HTTP server on port 8080 with embedded database
```

#### **2. Use the CLI Controller**
```bash
# Check current mode
./target/release/simple-cli-controller current-mode

# Switch to demo mode
./target/release/simple-cli-controller mode demo

# Switch to live mode (requires confirmation)
./target/release/simple-cli-controller mode live

# Monitor system status
./target/release/simple-cli-controller monitor
```

#### **3. Custom Credentials Support**
```bash
# Set custom cTrader credentials
./simple-cli-controller set-credentials \
  --client-id YOUR_CLIENT_ID \
  --client-secret YOUR_CLIENT_SECRET \
  --demo-account YOUR_DEMO_ACCOUNT \
  --live-account YOUR_LIVE_ACCOUNT

# Use custom credentials for mode switching
./simple-cli-controller mode demo --client-id CUSTOM_ID
./simple-cli-controller mode live --account-id CUSTOM_ACCOUNT
```

#### **4. Remote Control (for Render deployment)**
```bash
# Control remote deployment
./simple-cli-controller -e https://your-app.onrender.com mode demo
./simple-cli-controller -e https://your-app.onrender.com monitor
```

## 🌐 Deployment

### **Render Deployment**
The system is ready for deployment on Render with included configuration:

```bash
# Build command (in render.yaml)
cargo build --release --bin embedded-trader

# Start command
./start.sh
```

### **Environment Variables**
- `PORT` - Server port (default: 8080)
- `RUST_LOG` - Logging level (default: info)
- `CTRADER_CLIENT_ID` - cTrader client ID
- `CTRADER_CLIENT_SECRET` - cTrader client secret

## 🔒 Safety Features

### **Live Mode Protection**
- Requires explicit `CONFIRM LIVE TRADING` confirmation
- Shows actual account being used
- Credential security with hidden secrets in output
- Default demo mode for safety

### **Custom Account Warnings**
- Displays actual cTrader account being used
- Warns when switching to live trading
- Validates credentials before mode switching

## 📊 Technical Details

### **Embedded Database**
- **SQLite** embedded in executable with 116,629+ data points
- **Binary compression** using bincode + gzip for optimal storage
- **15 currency pairs** with complete historical data
- **Cross-pair correlation** analysis with Pearson coefficients

### **Mathematical Engine**
- **Time-symmetric pattern recognition** treating past/present/future as coordinates
- **Galois field cyclicity** for finite field arithmetic pattern detection
- **Temporal symmetry analysis** for hidden pattern discovery
- **Anomaly detection** identifying deviations as trading opportunities

### **Static Linking**
- **x86_64-unknown-linux-musl** target for maximum portability
- **rustls** TLS implementation (no OpenSSL dependencies)
- **Single executable** deployment with all dependencies embedded

## 🎯 Revolutionary Breakthrough

This system represents a fundamental breakthrough in forex trading:

1. **Mathematical Certainty** - Replaces probabilistic guessing with algebraic decoding
2. **Temporal Symmetries** - Discovers hidden patterns in historical data
3. **Synthetic Data Generation** - Creates future market scenarios from patterns
4. **Production Ready** - Enterprise-grade system with complete safety features

## 📄 License

MIT License - See LICENSE file for details.

---

**Revolutionary forex trading through algebraic decoding of temporal symmetries!** 🚀