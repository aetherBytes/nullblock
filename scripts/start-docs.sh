#!/bin/bash
# Start mdBook documentation server for NullBlock internal docs
# Port: 3001 (to avoid conflict with Erebus on 3000)

set -e

DOCS_DIR="${HOME}/nullblock/docs-internal"
PORT="${DOCS_PORT:-3001}"

echo "📚 Starting NullBlock Internal Documentation Server..."
echo "   Directory: ${DOCS_DIR}"
echo "   Port: ${PORT}"
echo ""

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo "❌ mdbook not found. Installing via cargo..."
    cargo install mdbook
fi

cd "${DOCS_DIR}"

# Build first to ensure it compiles
echo "📖 Building documentation..."
mdbook build

echo ""
echo "🚀 Starting server at http://localhost:${PORT}"
echo "   Press Ctrl+C to stop"
echo ""

# Serve with watch for live reload
mdbook serve --port "${PORT}" --open
