# NullBlock Content Service - Integration Test Guide

**Date:** 2026-02-06
**Status:** Infrastructure configured, awaiting Docker environment for full test

---

## Test Environment Setup

### Prerequisites Check
```bash
# Verify Docker is installed and running
docker --version
docker ps

# Verify justfile commands available
which just
```

---

## Phase 1: Infrastructure Startup

### 1.1 Start All Infrastructure
```bash
cd ~/nullblock
just start
```

**Expected Output:**
```
🐧 Starting services for Linux...
🚀 Starting all NullBlock infrastructure services...

📦 Creating Docker network for container communication...
  ✅ Network ready

📦 Creating persistent volumes...
  ✅ Volumes ready

📦 Starting PostgreSQL databases...
  ⏳ Waiting for Erebus PostgreSQL...
  ✅ Erebus PostgreSQL ready
  ⏳ Waiting for Agents PostgreSQL...
  ✅ Agents PostgreSQL ready
  ⏳ Waiting for Content PostgreSQL...
  ✅ Content PostgreSQL ready

📦 Starting Redis...
  ✅ Redis ready

📦 Starting Kafka...
  ✅ Kafka ready

🔄 Running database migrations...
📋 Step 4: Running Content Service database migrations...
  📄 Running 001_create_content_queue.sql...
  📄 Running 002_create_content_metrics.sql...
  📄 Running 003_create_content_templates.sql...
  ✅ Content migrations completed

✅ All infrastructure services are running!
```

### 1.2 Verify Content Database Container
```bash
docker ps --filter "name=nullblock-postgres-content"
```

**Expected:**
- Container `nullblock-postgres-content` running
- Port mapping: 0.0.0.0:5442->5432/tcp

### 1.3 Verify Database Created
```bash
docker exec nullblock-postgres-content psql -U postgres -l | grep nullblock_content
```

**Expected:**
```
 nullblock_content | postgres | UTF8     | ...
```

### 1.4 Verify Tables Created
```bash
docker exec nullblock-postgres-content psql -U postgres -d nullblock_content -c "\dt"
```

**Expected Tables:**
```
 public | content_metrics   | table | postgres
 public | content_queue     | table | postgres
 public | content_templates | table | postgres
```

### 1.5 Verify Indexes Created
```bash
docker exec nullblock-postgres-content psql -U postgres -d nullblock_content -c "\di"
```

**Expected Indexes:**
- `idx_content_queue_status`
- `idx_content_queue_theme`
- `idx_content_queue_scheduled`
- `idx_content_metrics_content`
- `idx_content_templates_theme`
- `idx_content_templates_active`

---

## Phase 2: Service Startup

### 2.1 Start Content Service (Manual)
```bash
cd ~/nullblock/svc/nullblock-content
cargo run --release
```

**Expected Output:**
```
📁 Loaded environment from .env.dev
🚀 Starting NullBlock Content Service
Connecting to database...
✅ Database connection pool established (20 connections)
📄 Loading templates from config/templates.json...
✅ Templates loaded: 5 themes, 13 variants
🎯 Event publisher: NoOpPublisher (no EVENT_ENDPOINT set)
🚀 Server listening on http://0.0.0.0:8002
```

### 2.2 Start Content Service (Via Script)
```bash
~/nullblock/scripts/start-content.sh
```

**Expected:**
- Auto-runs migrations
- Starts service
- Logs to `logs/content.log`

### 2.3 Verify Service Running
```bash
curl http://localhost:8002/health
```

**Expected Response:**
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
  -d '{"theme":"morning_insight","include_image":false}'
```

**Expected Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "theme": "morning_insight",
  "text": "Protocols don't sleep. Neither should your infrastructure.\n\nShip protocols, not promises.",
  "tags": ["infrastructure", "morning", "protocols"],
  "image_prompt": null,
  "status": "pending",
  "created_at": "2026-02-06T12:00:00Z"
}
```

**Validation:**
- ✅ UUID generated
- ✅ Text contains two parts (insight + reminder/tagline)
- ✅ Tags match theme
- ✅ Status is "pending"
- ✅ Timestamp is ISO8601

### 3.2 Generate Content (All Themes)
```bash
# Test all 5 themes
for theme in morning_insight progress_update educational eerie_fun community; do
  echo "Testing theme: $theme"
  curl -s -X POST http://localhost:8002/api/content/generate \
    -H "Content-Type: application/json" \
    -d "{\"theme\":\"$theme\",\"include_image\":false}" \
    | python3 -m json.tool
  echo ""
  sleep 1
done
```

**Expected:**
- 5 unique content pieces generated
- Each has appropriate tags for theme
- All have status "pending"

### 3.3 Generate Content with Image
```bash
curl -X POST http://localhost:8002/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{"theme":"eerie_fun","include_image":true}'
```

**Expected:**
- `image_prompt` field populated
- Image prompt mentions "retro-futuristic propaganda poster"
- Image prompt describes the content visually

### 3.4 List Pending Queue
```bash
curl -s "http://localhost:8002/api/content/queue?status=pending" | python3 -m json.tool
```

**Expected Response:**
```json
{
  "items": [
    {
      "id": "uuid",
      "theme": "morning_insight",
      "text": "...",
      "tags": [...],
      "status": "pending",
      ...
    }
  ],
  "total": 5
}
```

**Validation:**
- ✅ `total` matches number of items
- ✅ All items have status "pending"
- ✅ Items sorted by created_at (newest first)

### 3.5 Get Specific Content
```bash
# Replace with actual UUID from previous test
CONTENT_ID="550e8400-e29b-41d4-a716-446655440000"
curl -s "http://localhost:8002/api/content/queue/$CONTENT_ID" | python3 -m json.tool
```

**Expected:**
- Single content item returned
- All fields populated correctly

### 3.6 Update Content Status
```bash
CONTENT_ID="550e8400-e29b-41d4-a716-446655440000"
curl -X PUT "http://localhost:8002/api/content/queue/$CONTENT_ID" \
  -H "Content-Type: application/json" \
  -d '{"status":"approved"}'
```

**Expected:**
- Status changed to "approved"
- `updated_at` timestamp updated

### 3.7 List Approved Content
```bash
curl -s "http://localhost:8002/api/content/queue?status=approved" | python3 -m json.tool
```

**Expected:**
- Only approved items returned
- Previously approved item appears in list

### 3.8 Delete Content
```bash
CONTENT_ID="550e8400-e29b-41d4-a716-446655440000"
curl -X DELETE "http://localhost:8002/api/content/queue/$CONTENT_ID"
```

**Expected:**
- HTTP 204 No Content
- No response body
- Item no longer in queue

### 3.9 List Templates
```bash
curl -s "http://localhost:8002/api/content/templates" | python3 -m json.tool
```

**Expected:**
- Array of template configurations (if seeded)
- Empty array if no templates in DB

---

## Phase 4: Erebus Integration Test

### 4.1 Start Erebus
```bash
cd ~/nullblock/svc/erebus
cargo run --release
```

**Expected:**
- Erebus starts on port 3000
- Routes registered for `/api/content/*`

### 4.2 Test Proxy Health Check
```bash
curl http://localhost:3000/api/content/health
```

**Expected:**
- Same response as direct service call
- Proxied through Erebus

### 4.3 Test Proxy Generate
```bash
curl -X POST http://localhost:3000/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{"theme":"morning_insight","include_image":false}'
```

**Expected:**
- Content generated via Erebus proxy
- 30s timeout enforced
- Proper error handling if content service down

### 4.4 Test All Endpoints via Erebus
```bash
# Generate
curl -X POST http://localhost:3000/api/content/generate -H "Content-Type: application/json" -d '{"theme":"morning_insight","include_image":false}'

# Queue
curl "http://localhost:3000/api/content/queue?status=pending"

# Get
curl "http://localhost:3000/api/content/queue/{id}"

# Update
curl -X PUT "http://localhost:3000/api/content/queue/{id}" -H "Content-Type: application/json" -d '{"status":"approved"}'

# Delete
curl -X DELETE "http://localhost:3000/api/content/queue/{id}"

# Templates
curl "http://localhost:3000/api/content/templates"
```

**Expected:**
- All endpoints accessible through Erebus
- Responses match direct service calls

---

## Phase 5: Error Handling Tests

### 5.1 Invalid Theme
```bash
curl -X POST http://localhost:8002/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{"theme":"invalid_theme","include_image":false}'
```

**Expected:**
- HTTP 400 Bad Request
- Error message: "Invalid theme"

### 5.2 Missing Parameters
```bash
curl -X POST http://localhost:8002/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{}'
```

**Expected:**
- HTTP 400 Bad Request
- Error message about missing required fields

### 5.3 Invalid UUID
```bash
curl "http://localhost:8002/api/content/queue/not-a-uuid"
```

**Expected:**
- HTTP 400 Bad Request or 404 Not Found

### 5.4 Invalid Status Update
```bash
curl -X PUT "http://localhost:8002/api/content/queue/{valid-uuid}" \
  -H "Content-Type: application/json" \
  -d '{"status":"invalid_status"}'
```

**Expected:**
- HTTP 400 Bad Request
- Error message about invalid status

### 5.5 Non-existent Content
```bash
curl "http://localhost:8002/api/content/queue/00000000-0000-0000-0000-000000000000"
```

**Expected:**
- HTTP 404 Not Found

---

## Phase 6: Database Verification

### 6.1 Verify Content in Database
```bash
docker exec nullblock-postgres-content psql -U postgres -d nullblock_content \
  -c "SELECT id, theme, status, created_at FROM content_queue ORDER BY created_at DESC LIMIT 5;"
```

**Expected:**
- Shows recent content entries
- Status values: pending, approved, posted, failed, deleted
- Timestamps in UTC

### 6.2 Verify Queue Filtering
```bash
# Create content with different statuses
# Then verify database has correct counts

docker exec nullblock-postgres-content psql -U postgres -d nullblock_content \
  -c "SELECT status, COUNT(*) FROM content_queue GROUP BY status;"
```

**Expected:**
```
  status   | count
-----------+-------
 pending   |     3
 approved  |     1
 posted    |     0
```

---

## Phase 7: Content Quality Validation

### 7.1 Generate 10 Pieces (Variety Test)
```bash
for i in {1..10}; do
  curl -s -X POST http://localhost:8002/api/content/generate \
    -H "Content-Type: application/json" \
    -d '{"theme":"morning_insight","include_image":false}' \
    | python3 -c "import sys,json; print(json.load(sys.stdin)['text'])"
  echo ""
  sleep 1
done
```

**Validation:**
- ✅ Minimal duplication (should see different combinations)
- ✅ All content matches brand voice
- ✅ No Fallout/Vault-Tec references
- ✅ Infrastructure/protocol focus maintained

### 7.2 Tag Validation
```bash
# Get all content and check tags
curl -s "http://localhost:8002/api/content/queue" | python3 -c "
import sys, json
data = json.load(sys.stdin)
for item in data['items']:
    print(f\"{item['theme']}: {item['tags']}\")
"
```

**Expected Tags by Theme:**
- **morning_insight:** infrastructure, morning, protocols
- **progress_update:** progress, dev, building
- **educational:** educational, deepdive, agents
- **eerie_fun:** darkhumor, ai, autonomous
- **community:** community, engagement, poll

---

## Phase 8: Monitoring & Logs

### 8.1 Check Service Logs
```bash
tail -f ~/nullblock/svc/nullblock-content/logs/content.log
```

**Expected Log Entries:**
- Service startup
- Database connections
- Template loading
- Request logs (generate, queue operations)
- No error spam

### 8.2 Health Monitor
```bash
~/nullblock/scripts/monitor-content-health.sh
```

**Expected:**
- 5-second refresh
- Health status: HEALTHY
- Pending queue count displayed

---

## Phase 9: Dev Environment Test

### 9.1 Launch Full Dev Environment
```bash
just dev-mac  # or just dev-linux
```

**Expected:**
- Tmuxinator session starts
- All services launch in dedicated windows
- Content service window has 3 panes:
  1. Service logs
  2. Health monitor
  3. Log tail

### 9.2 Verify in Tmuxinator
- Switch to `nullblock-content` window
- Verify service is running
- Verify health monitor is refreshing
- Verify logs are streaming

---

## Success Criteria

### Infrastructure
- ✅ Database container running on port 5442
- ✅ 3 tables created (content_queue, content_metrics, content_templates)
- ✅ 6 indexes created
- ✅ Migrations run successfully

### Service
- ✅ Service starts on port 8002
- ✅ Health endpoint responds
- ✅ Templates load from JSON
- ✅ Database connection pool established

### API Functionality
- ✅ Generate content for all 5 themes
- ✅ Queue management (list, get, update, delete)
- ✅ Status updates working
- ✅ Filtering by status and theme
- ✅ Image prompts generate when requested

### Integration
- ✅ Erebus proxy routes working
- ✅ All 8 endpoints accessible via :3000
- ✅ Error handling working correctly

### Content Quality
- ✅ Brand voice maintained (cheerfully inevitable)
- ✅ No Fallout references
- ✅ Infrastructure focus present
- ✅ Good variety (minimal duplication)
- ✅ Correct tags per theme

---

## Cleanup

```bash
# Stop service
# Ctrl+C in service terminal

# Or via tmuxinator
tmux kill-session -t nullblock-dev

# Stop infrastructure
just term

# Wipe databases (if needed for fresh start)
just wipe-db
```

---

## Troubleshooting

### Service won't start
**Issue:** Database connection hangs
**Fix:**
```bash
# Check container is running
docker ps --filter "name=nullblock-postgres-content"

# Check database is ready
docker exec nullblock-postgres-content pg_isready -U postgres

# Restart infrastructure
just term && just start
```

### Migrations fail
**Issue:** "already exists" errors
**Fix:** These are expected on re-runs. Service handles them gracefully.

### No content variety
**Issue:** Same content generated repeatedly
**Fix:** Check templates.json is loaded correctly. Service should use random selection.

### Health endpoint fails
**Issue:** Connection refused
**Fix:**
```bash
# Check service is actually running
lsof -i :8002

# Check logs
tail ~/nullblock/svc/nullblock-content/logs/content.log
```

---

## Next Steps After Testing

1. **Event Publishing:** Add `EVENT_ENDPOINT` to .env.dev to enable HTTP event publishing
2. **Social Integration:** Add Twitter/X API keys for actual posting
3. **Template Seeding:** Seed templates into database for dynamic updates
4. **Metrics Collection:** Implement periodic metrics fetching from social platforms
5. **Scheduling:** Add cron-based content generation scheduling

---

**Test Status:** Ready for execution when Docker environment is available
**Infrastructure:** Fully configured and committed to repo
**Scripts:** All created and tested for syntax
