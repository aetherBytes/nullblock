#!/bin/bash
# Generate image for content using nano-banana-pro

if [ -z "$1" ]; then
    echo "Usage: ./generate-image.sh <content-id>"
    echo "Example: ./generate-image.sh MORNING_INSIGHT_1738541342539"
    exit 1
fi

CONTENT_ID="$1"
QUEUE_DIR="$(dirname "$0")/../content-queue"
PENDING_FILE="$QUEUE_DIR/pending/${CONTENT_ID}.json"
IMAGES_DIR="$QUEUE_DIR/images"

if [ ! -f "$PENDING_FILE" ]; then
    echo "Error: Content item $CONTENT_ID not found"
    exit 1
fi

# Extract image prompt from queue item
IMAGE_PROMPT=$(jq -r '.imagePrompt' "$PENDING_FILE")

if [ "$IMAGE_PROMPT" = "null" ] || [ -z "$IMAGE_PROMPT" ]; then
    echo "Error: No image prompt found for $CONTENT_ID"
    echo "Generate content with --image flag"
    exit 1
fi

# Create images directory if it doesn't exist
mkdir -p "$IMAGES_DIR"

# Generate image using nano-banana-pro
echo "Generating image for $CONTENT_ID..."
echo "Prompt: $IMAGE_PROMPT"
echo ""

# Use nano-banana-pro skill (assuming it's installed)
# Output will be saved with content ID
OUTPUT_FILE="$IMAGES_DIR/${CONTENT_ID}.png"

# Call nano-banana-pro (adjust path if needed)
nano-banana-pro generate "$IMAGE_PROMPT" --output "$OUTPUT_FILE" 2>&1 || {
    echo "Error: nano-banana-pro failed. Is it installed?"
    echo "Try: which nano-banana-pro"
    exit 1
}

if [ -f "$OUTPUT_FILE" ]; then
    echo "✅ Image generated: $OUTPUT_FILE"
    
    # Update queue item with image path
    TEMP_FILE=$(mktemp)
    jq ".imagePath = \"$OUTPUT_FILE\"" "$PENDING_FILE" > "$TEMP_FILE"
    mv "$TEMP_FILE" "$PENDING_FILE"
    
    echo "Updated queue item with image path"
else
    echo "❌ Image generation failed"
    exit 1
fi
