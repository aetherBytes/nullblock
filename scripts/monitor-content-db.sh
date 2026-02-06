#!/bin/bash

echo "🗄️  Content Database Monitor"
echo "============================"
echo ""

while true; do
  clear
  echo "🗄️  Content Database Monitor - $(date '+%H:%M:%S')"
  echo "============================"
  echo ""

  if docker exec nullblock-postgres-content psql -U postgres -d nullblock_content -c "SELECT COUNT(*) as total_content FROM content_queue;" 2>/dev/null; then
    echo ""
    echo "📊 Content by Status:"
    docker exec nullblock-postgres-content psql -U postgres -d nullblock_content -c "SELECT status, COUNT(*) as count FROM content_queue GROUP BY status ORDER BY count DESC;" 2>/dev/null
    echo ""
    echo "🎯 Content by Theme:"
    docker exec nullblock-postgres-content psql -U postgres -d nullblock_content -c "SELECT theme, COUNT(*) as count FROM content_queue GROUP BY theme ORDER BY count DESC;" 2>/dev/null
    echo ""
    echo "📅 Recent Content:"
    docker exec nullblock-postgres-content psql -U postgres -d nullblock_content -c "SELECT id, theme, status, created_at FROM content_queue ORDER BY created_at DESC LIMIT 5;" 2>/dev/null
  else
    echo "❌ Database not available"
    echo "   Start with: just start"
  fi

  echo ""
  echo "Press Ctrl+C to exit"
  sleep 5
done
