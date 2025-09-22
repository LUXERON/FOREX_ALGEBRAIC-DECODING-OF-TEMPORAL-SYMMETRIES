@echo off
echo.
echo ╔═══════════════════════════════════════════════════════════════════════════════════╗
echo ║                                                                                   ║
echo ║    🧪 FOREX WEBSOCKET DEPLOYMENT TEST 🧪                                        ║
echo ║         Testing Render Deployment Connectivity                                   ║
echo ║                                                                                   ║
echo ╚═══════════════════════════════════════════════════════════════════════════════════╝
echo.

set RENDER_URL=https://forex-websocket-trader.onrender.com
set WS_URL=wss://forex-websocket-trader.onrender.com/ws

echo 🔍 Testing HTTP endpoints...
echo.

echo 📡 Testing /health endpoint:
curl -s %RENDER_URL%/health
echo.
echo.

echo 📊 Testing /status endpoint:
curl -s %RENDER_URL%/status
echo.
echo.

echo 💱 Testing /pairs endpoint:
curl -s %RENDER_URL%/pairs
echo.
echo.

echo 🎮 Testing WebSocket CLI Controller:
echo.
echo Starting interactive WebSocket CLI...
echo Use these commands once connected:
echo   status          - Get system status
echo   start DEMO      - Start trading in DEMO mode
echo   pairs           - List currency pairs
echo   help            - Show all commands
echo   quit            - Exit
echo.

.\target\release\websocket-cli.exe --url %WS_URL%

echo.
echo 🎉 Test completed!
pause
