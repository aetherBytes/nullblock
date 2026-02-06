# NullBlock Content Service - Test Plan

## Test Status: Build Verified ✓

**Date:** 2026-02-05
**Build:** Success (8.46s)
**Binary:** `/target/release/nullblock-content`

---

## Phase 1: Build & Configuration Tests ✓

### 1.1 Compilation Test
```bash
cd /home/sagej/nullblock/svc/nullblock-content
cargo build --release
```
**Result:** ✅ Success with 10 warnings (unused methods - expected)

### 1.2 Environment Loading
```bash
./target/release/nullblock-content
```
**Result:** ✅ Loads .env.dev, starts service, attempts DB connection

### 1.3 Configuration Validation
- ✅ PORT: 8002
- ✅ SERVICE_NAME: nullblock-content
- ✅ DATABASE_URL: postgresql://postgres:password@localhost:5432/nullblock_content
- ✅ TEMPLATES_PATH: config/templates.json

---

## Phase 2: Database Tests (Requires PostgreSQL)

### Prerequisites
```bash
# Start PostgreSQL (Docker or local)
just start  # Starts all infrastructure

# Create database
createdb nullblock_content

# Run migrations
cd /home/sagej/nullblock/svc/nullblock-content
psql -d nullblock_content -f migrations/001_create_content_queue.sql
psql -d nullblock_content -f migrations/002_create_content_metrics.sql
psql -d nullblock_content -f migrations/003_create_content_templates.sql
```

### 2.1 Database Connection
```bash
# Should connect successfully
./target/release/nullblock-content
```
**Expected:** Service starts on port 8002

### 2.2 Health Check
```bash
curl http://localhost:8002/health
```
**Expected:**
```json
{
  "status": "healthy",
  "service": "nullblock-content",
  "version": "0.1.0"
}
```

---

## Phase 3: API Endpoint Tests

### 3.1 Generate Content (Morning Insight)
```bash
curl -X POST http://localhost:8002/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{
    "theme": "morning_insight",
    "include_image": false
  }'
```
**Expected:** UUID + generated text + tags + status=pending

### 3.2 Generate Content with Image
```bash
curl -X POST http://localhost:8002/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{
    "theme": "eerie_fun",
    "include_image": true
  }'
```
**Expected:** UUID + generated text + image_prompt (retro-futuristic style)

### 3.3 List All Themes
```bash
for theme in morning_insight progress_update educational eerie_fun community; do
  echo "Testing theme: $theme"
  curl -X POST http://localhost:8002/api/content/generate \
    -H "Content-Type: application/json" \
    -d "{\"theme\":\"$theme\",\"include_image\":false}"
  echo ""
done
```
**Expected:** 5 unique content pieces, all with status=pending

### 3.4 Queue Management
```bash
# List pending content
curl "http://localhost:8002/api/content/queue?status=pending"

# Get specific content (replace UUID)
curl "http://localhost:8002/api/content/queue/<uuid>"

# Update status to approved
curl -X PUT "http://localhost:8002/api/content/queue/<uuid>" \
  -H "Content-Type: application/json" \
  -d '{"status":"approved"}'

# List approved content
curl "http://localhost:8002/api/content/queue?status=approved"
```

### 3.5 Templates
```bash
curl "http://localhost:8002/api/content/templates"
```
**Expected:** Array of template configurations (if seeded)

### 3.6 Delete Content
```bash
curl -X DELETE "http://localhost:8002/api/content/queue/<uuid>"
```
**Expected:** 204 No Content

---

## Phase 4: Erebus Integration Tests

### Prerequisites
```bash
# Start Erebus on port 3000
cd /home/sagej/nullblock/svc/erebus
cargo run --release
```

### 4.1 Proxy Health Check
```bash
curl http://localhost:3000/api/content/health
```
**Expected:** Same response as direct service

### 4.2 Proxy Generate
```bash
curl -X POST http://localhost:3000/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{
    "theme": "morning_insight",
    "include_image": false
  }'
```
**Expected:** Proxied response from content service

### 4.3 All Endpoints via Erebus
```bash
# Test all 8 endpoints through Erebus proxy
curl http://localhost:3000/api/content/queue
curl http://localhost:3000/api/content/templates
# ... etc
```

---

## Phase 5: Event Publishing Tests

### 5.1 HTTP Event Publisher
```bash
# Set EVENT_ENDPOINT in .env.dev
echo "EVENT_ENDPOINT=http://localhost:9000/events" >> .env.dev

# Restart service
./target/release/nullblock-content

# Generate content - should publish event
curl -X POST http://localhost:8002/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{"theme":"morning_insight","include_image":false}'
```
**Expected:** Check logs for event publishing (or set up event receiver)

### 5.2 Event Payload Validation
**Expected events:**
- `content.generated` - On POST /api/content/generate
- `content.posted` - On successful social media post (future)
- `content.failed` - On generation/posting errors

---

## Phase 6: Content Quality Tests

### 6.1 Brand Voice Validation
**Manual review of generated content:**
- ✅ Cheerfully inevitable tone
- ✅ Professionally apocalyptic style
- ✅ Infrastructure/protocol focus
- ❌ No Fallout/Vault-Tec references
- ❌ No financial advice or token shilling

### 6.2 Template Variety
```bash
# Generate 10 pieces of same theme
for i in {1..10}; do
  curl -X POST http://localhost:8002/api/content/generate \
    -H "Content-Type: application/json" \
    -d '{"theme":"morning_insight","include_image":false}' \
    | jq '.text'
done
```
**Expected:** Minimal duplication, varied content

### 6.3 Tag Validation
**Each theme should have appropriate tags:**
- morning_insight: infrastructure, morning, protocols
- progress_update: progress, dev, building
- educational: educational, deepdive, agents
- eerie_fun: darkhumor, ai, autonomous
- community: community, engagement, poll

---

## Phase 7: Error Handling Tests

### 7.1 Invalid Theme
```bash
curl -X POST http://localhost:8002/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{"theme":"invalid_theme","include_image":false}'
```
**Expected:** 400 Bad Request with error message

### 7.2 Missing Parameters
```bash
curl -X POST http://localhost:8002/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{}'
```
**Expected:** 400 Bad Request

### 7.3 Invalid UUID
```bash
curl "http://localhost:8002/api/content/queue/not-a-uuid"
```
**Expected:** 400 Bad Request or 404 Not Found

### 7.4 Invalid Status Update
```bash
curl -X PUT "http://localhost:8002/api/content/queue/<uuid>" \
  -H "Content-Type: application/json" \
  -d '{"status":"invalid_status"}'
```
**Expected:** 400 Bad Request

---

## Phase 8: Load Tests (Optional)

### 8.1 Concurrent Generation
```bash
# Generate 100 pieces concurrently
seq 1 100 | xargs -P 10 -I {} curl -X POST http://localhost:8002/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{"theme":"morning_insight","include_image":false}'
```
**Expected:** All succeed, DB handles concurrent writes

### 8.2 Queue Pagination
```bash
# Generate 100 pieces, then paginate
curl "http://localhost:8002/api/content/queue?limit=10&offset=0"
curl "http://localhost:8002/api/content/queue?limit=10&offset=10"
```

---

## Known Limitations (Current Environment)

**Cannot test:**
- ❌ Database operations (PostgreSQL not running)
- ❌ Erebus proxy (Erebus not started)
- ❌ Event publishing (no receiver endpoint)

**Can test:**
- ✅ Build success
- ✅ Configuration loading
- ✅ Service startup sequence

---

## Next Steps

**To fully test:**
1. Start PostgreSQL on port 5432
2. Run migrations (001, 002, 003)
3. Start nullblock-content service
4. Run test commands from Phases 3-8
5. Start Erebus and test proxy routes (Phase 4)

**Recommended:**
- Set up automated integration tests with testcontainers
- Add unit tests for generator logic
- Mock database for CI/CD

---

## Quick Test Script (When DB Available)

```bash
#!/bin/bash
# test-content-service.sh

# Start service
./target/release/nullblock-content &
SERVICE_PID=$!
sleep 2

# Health check
echo "Testing health endpoint..."
curl -s http://localhost:8002/health | jq '.'

# Generate content for all themes
echo -e "\nGenerating content for all themes..."
for theme in morning_insight progress_update educational eerie_fun community; do
  echo "Theme: $theme"
  curl -s -X POST http://localhost:8002/api/content/generate \
    -H "Content-Type: application/json" \
    -d "{\"theme\":\"$theme\",\"include_image\":false}" \
    | jq '.text'
done

# List queue
echo -e "\nListing queue..."
curl -s "http://localhost:8002/api/content/queue?status=pending" | jq '.total'

# Cleanup
kill $SERVICE_PID
```

---

**Status:** BUILD VERIFIED - Database tests pending infrastructure setup
