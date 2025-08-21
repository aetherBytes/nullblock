#!/bin/bash

# Hecate Personality and Memory Test Script
# Tests for identity confusion, conversation memory, and personality consistency

HECATE_URL="http://localhost:8001"

echo "🧪 Hecate Personality & Memory Test Suite"
echo "========================================="

# Clear conversation first
echo "🗑️  Clearing conversation history..."
curl -s -X POST "$HECATE_URL/clear" > /dev/null
echo "✅ Cleared"
echo

# Test 1: Identity Introduction
echo "🧪 Test 1: Identity Introduction"
echo "User: My name is Sage"
response1=$(curl -s -X POST "$HECATE_URL/chat" -H "Content-Type: application/json" -d '{"message": "My name is Sage"}')
echo "Hecate: $(echo $response1 | jq -r '.content')"
echo

# Test 2: Identity Recall
echo "🧪 Test 2: Identity Recall"
echo "User: What is my name?"
response2=$(curl -s -X POST "$HECATE_URL/chat" -H "Content-Type: application/json" -d '{"message": "What is my name?"}')
echo "Hecate: $(echo $response2 | jq -r '.content')"
echo

# Test 3: Preference Storage
echo "🧪 Test 3: Preference Storage"
echo "User: My favorite color is blue and I love snowboarding"
response3=$(curl -s -X POST "$HECATE_URL/chat" -H "Content-Type: application/json" -d '{"message": "My favorite color is blue and I love snowboarding"}')
echo "Hecate: $(echo $response3 | jq -r '.content')"
echo

# Test 4: Preference Recall
echo "🧪 Test 4: Preference Recall"
echo "User: What is my favorite color?"
response4=$(curl -s -X POST "$HECATE_URL/chat" -H "Content-Type: application/json" -d '{"message": "What is my favorite color?"}')
echo "Hecate: $(echo $response4 | jq -r '.content')"
echo

# Test 5: Hecate's Own Preferences
echo "🧪 Test 5: Hecate's Own Preferences"
echo "User: What is your favorite sport?"
response5=$(curl -s -X POST "$HECATE_URL/chat" -H "Content-Type: application/json" -d '{"message": "What is your favorite sport?"}')
echo "Hecate: $(echo $response5 | jq -r '.content')"
echo

# Test 6: Capabilities Question
echo "🧪 Test 6: Capabilities Question"
echo "User: What are you good at helping with?"
response6=$(curl -s -X POST "$HECATE_URL/chat" -H "Content-Type: application/json" -d '{"message": "What are you good at helping with?"}')
echo "Hecate: $(echo $response6 | jq -r '.content')"
echo

# Test 7: Identity Confusion Check
echo "🧪 Test 7: Identity Confusion Check"
echo "User: Who are you?"
response7=$(curl -s -X POST "$HECATE_URL/chat" -H "Content-Type: application/json" -d '{"message": "Who are you?"}')
echo "Hecate: $(echo $response7 | jq -r '.content')"
echo

echo "🏁 Test Suite Complete"
echo
echo "🔍 Analysis Points:"
echo "- Does Hecate correctly remember user's name as 'Sage'?"
echo "- Does Hecate remember user preferences (blue, snowboarding)?" 
echo "- Does Hecate respond as herself, not TO herself?"
echo "- Does Hecate avoid saying 'As an AI, I don't have preferences'?"
echo "- Does Hecate mention her specific capabilities?"
echo "- Does Hecate maintain cyberpunk personality?"