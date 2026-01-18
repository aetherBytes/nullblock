#!/bin/bash
# ArbFarm Event Stream Monitor (SSE format)

echo "📡 ArbFarm Events"
echo "Waiting for service..."

# Wait for service to be available
while ! curl -s http://localhost:9007/health > /dev/null 2>&1; do
  sleep 3
done

echo "✅ Service is up, connecting to event stream..."
sleep 2

while true; do
  # Try to connect to SSE stream
  # SSE format: "event: <topic>" followed by "data: <json>"
  curl -sN http://localhost:9007/events/stream 2>/dev/null | while read line; do
    # Only process lines starting with "data:"
    if [[ "$line" == data:* ]]; then
      # Strip "data: " prefix and parse JSON
      json="${line#data: }"
      ts=$(echo "$json" | jq -r '.timestamp[11:19]' 2>/dev/null)
      evt=$(echo "$json" | jq -r '.event_type' 2>/dev/null)
      topic=$(echo "$json" | jq -r '.topic' 2>/dev/null)

      if [ -n "$ts" ] && [ "$ts" != "null" ] && [ -n "$evt" ] && [ "$evt" != "null" ]; then
        # Color code by event type
        case "$evt" in
          *detected*)
            echo -e "\033[0;36m$ts\033[0m | \033[1;33m$evt\033[0m"
            ;;
          *executed*|*success*)
            echo -e "\033[0;36m$ts\033[0m | \033[1;32m$evt\033[0m"
            ;;
          *failed*|*error*)
            echo -e "\033[0;36m$ts\033[0m | \033[1;31m$evt\033[0m"
            ;;
          *)
            echo "$ts | $evt"
            ;;
        esac
      fi
    fi
  done

  # Connection lost or failed - wait and retry
  echo "$(date '+%H:%M:%S') | Connection lost, retrying in 5s..."
  sleep 5
done
