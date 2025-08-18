#!/bin/bash

# LM Studio Startup Script

echo "🚀 Starting LM Studio..."
lms status

echo "📦 Loading Gemma3 270M..."
lms load gemma-3-270m-it-mlx -y

echo "🔧 Model loaded successfully! Starting API server..."
lms server start

echo "✅ LM Studio server started successfully!"
echo "🚪 Exiting startup pane..."
exit