#!/bin/bash
set -e

if [ -n "$DATABASE_URL" ] && [ -d "/app/migrations" ]; then
    echo "Running Engrams migrations..."
    for f in /app/migrations/*.sql; do
        echo "Applying $(basename $f)..."
        psql "$DATABASE_URL" -f "$f" 2>&1 | grep -v "already exists" || true
    done
    echo "Migrations complete."
fi

exec "$@"
