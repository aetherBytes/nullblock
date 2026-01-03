#!/bin/bash
# Frontend (Hecate) startup script for tmuxinator

echo "🎨 Starting Frontend (Hecate React App)..."
echo "=============================================="

cd ~/nullblock/svc/hecate || exit 1

echo ""
echo "📋 Environment Info:"
echo "   Node: $(node --version)"
echo "   npm:  $(npm --version)"
echo ""

echo "🔗 API Endpoints:"
export VITE_PROTOCOLS_API_URL=http://localhost:8001
export VITE_A2A_API_URL=http://localhost:8001
export VITE_EREBUS_API_URL=http://localhost:3000
export VITE_HECATE_API_URL=http://localhost:9003
echo "   VITE_EREBUS_API_URL=$VITE_EREBUS_API_URL"
echo "   VITE_PROTOCOLS_API_URL=$VITE_PROTOCOLS_API_URL"
echo "   VITE_HECATE_API_URL=$VITE_HECATE_API_URL"
echo ""

echo "🔧 Chrome DevTools MCP:"
echo "   Debug port: 9222"
echo "   Profile: /tmp/chrome-nullblock-dev"
echo ""

echo "🧹 Clearing Vite cache..."
rm -rf node_modules/.vite

echo "📦 Installing/updating npm dependencies..."
npm install --silent
echo ""

echo "⏳ Waiting for backend services..."
for i in {1..30}; do
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        echo "✅ Erebus ready"
        break
    fi
    sleep 1
done
echo ""

echo "🚀 Starting Vite dev server..."
npm run develop
