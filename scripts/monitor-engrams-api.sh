#!/bin/bash
echo "🔬 Engram API Testing & Monitoring"
echo "Interactive API testing for engram endpoints..."
echo ""
echo "📡 Engram Endpoints via Erebus (Port 3000):"
echo "  POST   /api/engrams                - Create engram"
echo "  GET    /api/engrams                - List engrams"
echo "  GET    /api/engrams/:id            - Get by ID"
echo "  PUT    /api/engrams/:id            - Update engram"
echo "  DELETE /api/engrams/:id            - Delete engram"
echo "  GET    /api/engrams/wallet/:wallet - Get by wallet"
echo "  POST   /api/engrams/search         - Search engrams"
echo "  POST   /api/engrams/:id/fork       - Fork engram"
echo "  POST   /api/engrams/:id/publish    - Publish engram"
echo ""
echo "🧪 Running API health checks..."
while true; do
  echo ""
  echo "$(date '+%Y-%m-%d %H:%M:%S') [ENGRAM] API Health Check..."

  # Direct engram service health
  if curl -s --max-time 3 "http://localhost:9004/health" > /dev/null 2>&1; then
    echo "✅ Engram Service (9004) - Healthy"
    curl -s "http://localhost:9004/health" | jq . 2>/dev/null || echo "  (raw response)"
  else
    echo "❌ Engram Service (9004) - Not responding"
  fi

  # Engram via Erebus
  if curl -s --max-time 3 "http://localhost:3000/api/engrams/health" > /dev/null 2>&1; then
    echo "✅ Engram via Erebus (3000) - Healthy"
  else
    echo "⚠️  Engram via Erebus (3000) - Not responding (Erebus may not be running)"
  fi

  # Engram count
  engram_response=$(curl -s --max-time 3 "http://localhost:9004/engrams?limit=1" 2>/dev/null)
  if [ ! -z "$engram_response" ]; then
    total=$(echo "$engram_response" | jq '.total // 0' 2>/dev/null || echo "0")
    echo "📊 Total engrams in database: $total"
  fi

  sleep 45
done
