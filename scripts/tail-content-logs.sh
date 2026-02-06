#!/usr/bin/env bash

LOG_FILE="$HOME/nullblock/svc/nullblock-content/logs/content.log"

echo "📜 Content Service Logs"
echo "======================="
echo ""

if [ -f "$LOG_FILE" ]; then
  tail -f "$LOG_FILE"
else
  echo "⏳ Waiting for log file to be created..."
  echo "Log file: $LOG_FILE"
  while [ ! -f "$LOG_FILE" ]; do
    sleep 1
  done
  tail -f "$LOG_FILE"
fi
