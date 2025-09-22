#!/usr/bin/env python3
"""
🚀 FOREX WEBSOCKET TRADER - PYTHON WRAPPER FOR RENDER DEPLOYMENT

This Python wrapper runs the pre-built Rust executable on Render.
The executable contains the complete WebSocket trading system.
"""

import os
import sys
import subprocess
import signal
import time
from pathlib import Path

def print_banner():
    print("""
╔═══════════════════════════════════════════════════════════════════════════════════╗
║                                                                                   ║
║    🚀 FOREX WEBSOCKET TRADER - PYTHON WRAPPER 🚀                                ║
║         Running Pre-built Rust Executable on Render                              ║
║                                                                                   ║
╚═══════════════════════════════════════════════════════════════════════════════════╝
""")

def find_executable():
    """Find the websocket-trader executable"""
    current_dir = Path(__file__).parent
    possible_paths = [
        current_dir / "websocket-trader",
        current_dir / "websocket-trader.exe",
        current_dir / "../target/release/websocket-trader",
        current_dir / "../target/release/websocket-trader.exe",
    ]
    
    for path in possible_paths:
        if path.exists():
            print(f"✅ Found executable: {path}")
            return str(path)
    
    print("❌ ERROR: websocket-trader executable not found!")
    print("📋 Searched paths:")
    for path in possible_paths:
        print(f"   - {path} (exists: {path.exists()})")
    
    print("\n📁 Current directory contents:")
    for item in current_dir.iterdir():
        print(f"   - {item.name}")
    
    return None

def make_executable(path):
    """Make the file executable on Unix systems"""
    try:
        os.chmod(path, 0o755)
        print(f"🔧 Made {path} executable")
    except Exception as e:
        print(f"⚠️  Could not make executable: {e}")

def run_executable():
    """Run the websocket-trader executable"""
    executable_path = find_executable()
    
    if not executable_path:
        print("💡 Creating a fallback HTTP server...")
        create_fallback_server()
        return
    
    # Make executable if on Unix
    if os.name != 'nt':
        make_executable(executable_path)
    
    # Set environment variables
    env = os.environ.copy()
    env.update({
        'PORT': str(os.getenv('PORT', '10000')),
        'RUST_LOG': 'info',
        'TRADING_MODE': 'DEMO'
    })
    
    print(f"🚀 Starting Forex WebSocket Trader...")
    print(f"📡 Port: {env['PORT']}")
    print(f"🎯 Mode: {env['TRADING_MODE']}")
    print(f"📝 Log Level: {env['RUST_LOG']}")
    
    try:
        # Start the executable
        process = subprocess.Popen(
            [executable_path],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            universal_newlines=True,
            bufsize=1
        )
        
        print(f"✅ Process started with PID: {process.pid}")
        
        # Stream output
        for line in process.stdout:
            print(f"[TRADER] {line.rstrip()}")
            
    except FileNotFoundError:
        print(f"❌ Executable not found or not executable: {executable_path}")
        create_fallback_server()
    except Exception as e:
        print(f"❌ Error starting executable: {e}")
        create_fallback_server()

def create_fallback_server():
    """Create a simple fallback HTTP server"""
    print("🔄 Starting fallback HTTP server...")
    
    try:
        from http.server import HTTPServer, BaseHTTPRequestHandler
        import json
        from datetime import datetime
        
        class FallbackHandler(BaseHTTPRequestHandler):
            def do_GET(self):
                if self.path == '/health':
                    self.send_response(200)
                    self.send_header('Content-type', 'application/json')
                    self.end_headers()
                    response = {
                        'status': 'fallback',
                        'message': 'Python wrapper fallback server',
                        'executable_found': False,
                        'timestamp': datetime.now().isoformat()
                    }
                    self.wfile.write(json.dumps(response).encode())
                elif self.path == '/status':
                    self.send_response(200)
                    self.send_header('Content-type', 'application/json')
                    self.end_headers()
                    response = {
                        'service': 'forex-websocket-trader',
                        'wrapper': 'python',
                        'executable': 'missing',
                        'port': os.getenv('PORT', '10000'),
                        'timestamp': datetime.now().isoformat()
                    }
                    self.wfile.write(json.dumps(response).encode())
                else:
                    self.send_response(404)
                    self.end_headers()
                    self.wfile.write(b'Not Found')
            
            def log_message(self, format, *args):
                print(f"[FALLBACK] {format % args}")
        
        port = int(os.getenv('PORT', '10000'))
        server = HTTPServer(('0.0.0.0', port), FallbackHandler)
        print(f"🌐 Fallback server running on port {port}")
        server.serve_forever()
        
    except Exception as e:
        print(f"❌ Could not start fallback server: {e}")
        sys.exit(1)

def signal_handler(signum, frame):
    """Handle shutdown signals"""
    print(f"\n📴 Received signal {signum}, shutting down gracefully...")
    sys.exit(0)

def main():
    """Main entry point"""
    print_banner()
    
    # Set up signal handlers
    signal.signal(signal.SIGTERM, signal_handler)
    signal.signal(signal.SIGINT, signal_handler)
    
    # Run the executable
    run_executable()

if __name__ == "__main__":
    main()
