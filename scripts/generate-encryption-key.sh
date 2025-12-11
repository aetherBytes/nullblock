#!/bin/bash
set -e

echo "🔐 Generating 256-bit AES encryption master key..."
echo ""

KEY=$(openssl rand -hex 32)

echo "✅ Master key generated successfully!"
echo ""
echo "Add this to your .env.dev file (NEVER commit to git):"
echo ""
echo "ENCRYPTION_MASTER_KEY=$KEY"
echo ""
echo "⚠️  IMPORTANT: Keep this key secure. If lost, all encrypted API keys will be unrecoverable."
