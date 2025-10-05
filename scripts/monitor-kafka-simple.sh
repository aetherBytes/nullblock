#!/bin/bash
# Simple Kafka monitoring for agents

echo "📨 Monitoring Kafka Event Streaming"
echo "Simple topic list and log monitoring..."
echo ""

# Show topics
echo "📋 Kafka Topics:"
docker exec nullblock-kafka kafka-topics --list --bootstrap-server localhost:9092 2>/dev/null || echo "❌ Kafka not available"
echo ""

# Monitor logs
echo "📊 Monitoring agents database activity..."
tail -f ~/nullblock/logs/agents-db.log 2>/dev/null || echo "⚠️ Waiting for logs..."
