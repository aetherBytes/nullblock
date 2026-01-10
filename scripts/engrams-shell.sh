#!/bin/bash
echo "🔧 Engram Interactive Shell"
echo "Use this pane for manual API testing..."
echo ""
echo "📝 Example Commands:"
echo ""
echo "# Create an engram:"
echo 'curl -X POST http://localhost:3000/api/engrams \'
echo '  -H "Content-Type: application/json" \'
echo '  -d '"'"'{"wallet_address":"0x123","engram_type":"persona","key":"test.persona","content":{"name":"Test"},"tags":["test"]}'"'"' | jq .'
echo ""
echo "# List engrams:"
echo "curl -s http://localhost:3000/api/engrams | jq ."
echo ""
echo "# Search engrams:"
echo 'curl -X POST http://localhost:3000/api/engrams/search \'
echo '  -H "Content-Type: application/json" \'
echo '  -d '"'"'{"engram_type":"persona","tags":["test"]}'"'"' | jq .'
echo ""
echo "# Get by wallet:"
echo "curl -s http://localhost:3000/api/engrams/wallet/0x123 | jq ."
echo ""
echo "Ready for interactive testing..."
cd ~/nullblock
bash
