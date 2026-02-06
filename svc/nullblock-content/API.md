# NullBlock Content Service API

**Version:** 0.1.0
**Base URL:** `http://localhost:3000/api/content` (via Erebus)
**Direct URL:** `http://localhost:8002/api/content` (development only)

## Authentication

Currently no authentication required. Future versions will integrate with Erebus wallet authentication.

## Endpoints

### Health Check

Check service health and version.

**Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "healthy",
  "service": "nullblock-content",
  "version": "0.1.0"
}
```

**Status Codes:**
- `200 OK` - Service healthy

---

### Generate Content

Generate themed content using the template system.

**Endpoint:** `POST /api/content/generate`

**Request Body:**
```json
{
  "theme": "morning_insight",
  "include_image": false
}
```

**Parameters:**
- `theme` (string, required) - One of: `morning_insight`, `progress_update`, `educational`, `eerie_fun`, `community`
- `include_image` (boolean, required) - Whether to generate an image prompt

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "theme": "morning_insight",
  "text": "Protocols don't sleep. Neither should your infrastructure.\n\nShip protocols, not promises.",
  "tags": ["infrastructure", "morning", "protocols"],
  "image_prompt": null,
  "status": "pending",
  "created_at": "2026-02-02T12:00:00Z"
}
```

**Status Codes:**
- `200 OK` - Content generated successfully
- `400 Bad Request` - Invalid theme or parameters
- `500 Internal Server Error` - Database or generation error

**Example:**
```bash
curl -X POST http://localhost:3000/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{"theme":"morning_insight","include_image":false}'
```

---

### List Content Queue

Retrieve content from the queue with optional filtering.

**Endpoint:** `GET /api/content/queue`

**Query Parameters:**
- `status` (string, optional) - Filter by status: `pending`, `approved`, `posted`, `failed`, `deleted`
- `theme` (string, optional) - Filter by theme
- `limit` (integer, optional) - Number of items to return (default: 50)
- `offset` (integer, optional) - Pagination offset (default: 0)

**Response:**
```json
{
  "items": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "theme": "morning_insight",
      "text": "Content text here",
      "tags": ["infrastructure", "morning"],
      "image_prompt": null,
      "image_path": null,
      "status": "pending",
      "scheduled_at": null,
      "posted_at": null,
      "tweet_url": null,
      "metadata": {},
      "created_at": "2026-02-02T12:00:00Z",
      "updated_at": "2026-02-02T12:00:00Z"
    }
  ],
  "total": 1
}
```

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Invalid query parameters
- `500 Internal Server Error` - Database error

**Examples:**
```bash
# List all pending content
curl "http://localhost:3000/api/content/queue?status=pending"

# List by theme with pagination
curl "http://localhost:3000/api/content/queue?theme=morning_insight&limit=10&offset=0"

# List all posted content
curl "http://localhost:3000/api/content/queue?status=posted"
```

---

### Get Content by ID

Retrieve a specific content item.

**Endpoint:** `GET /api/content/queue/:id`

**Parameters:**
- `id` (UUID, required) - Content ID

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "theme": "morning_insight",
  "text": "Content text here",
  "tags": ["infrastructure", "morning"],
  "image_prompt": null,
  "image_path": null,
  "status": "pending",
  "scheduled_at": null,
  "posted_at": null,
  "tweet_url": null,
  "metadata": {},
  "created_at": "2026-02-02T12:00:00Z",
  "updated_at": "2026-02-02T12:00:00Z"
}
```

**Status Codes:**
- `200 OK` - Content found
- `404 Not Found` - Content not found
- `500 Internal Server Error` - Database error

**Example:**
```bash
curl "http://localhost:3000/api/content/queue/550e8400-e29b-41d4-a716-446655440000"
```

---

### Update Content Status

Update the status of a content item.

**Endpoint:** `PUT /api/content/queue/:id`

**Parameters:**
- `id` (UUID, required) - Content ID

**Request Body:**
```json
{
  "status": "approved"
}
```

**Parameters:**
- `status` (string, required) - New status: `pending`, `approved`, `posted`, `failed`, `deleted`

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "theme": "morning_insight",
  "text": "Content text here",
  "tags": ["infrastructure", "morning"],
  "image_prompt": null,
  "image_path": null,
  "status": "approved",
  "scheduled_at": null,
  "posted_at": null,
  "tweet_url": null,
  "metadata": {},
  "created_at": "2026-02-02T12:00:00Z",
  "updated_at": "2026-02-02T12:05:00Z"
}
```

**Status Codes:**
- `200 OK` - Status updated
- `404 Not Found` - Content not found
- `500 Internal Server Error` - Database error

**Example:**
```bash
curl -X PUT "http://localhost:3000/api/content/queue/550e8400-e29b-41d4-a716-446655440000" \
  -H "Content-Type: application/json" \
  -d '{"status":"approved"}'
```

---

### Delete Content

Delete a content item from the queue.

**Endpoint:** `DELETE /api/content/queue/:id`

**Parameters:**
- `id` (UUID, required) - Content ID

**Response:**
- `204 No Content` - Successfully deleted

**Status Codes:**
- `204 No Content` - Content deleted
- `404 Not Found` - Content not found
- `500 Internal Server Error` - Database error

**Example:**
```bash
curl -X DELETE "http://localhost:3000/api/content/queue/550e8400-e29b-41d4-a716-446655440000"
```

---

### Get Content Metrics

Retrieve engagement metrics for posted content.

**Endpoint:** `GET /api/content/metrics/:id`

**Parameters:**
- `id` (UUID, required) - Content ID

**Response:**
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "content_id": "550e8400-e29b-41d4-a716-446655440000",
  "likes": 42,
  "retweets": 15,
  "replies": 8,
  "impressions": 1250,
  "engagement_rate": 5.2,
  "fetched_at": "2026-02-03T12:00:00Z",
  "created_at": "2026-02-03T12:00:00Z"
}
```

**Status Codes:**
- `200 OK` - Metrics found
- `404 Not Found` - No metrics for content
- `500 Internal Server Error` - Database error

**Example:**
```bash
curl "http://localhost:3000/api/content/metrics/550e8400-e29b-41d4-a716-446655440000"
```

---

### List Templates

Retrieve active content templates.

**Endpoint:** `GET /api/content/templates`

**Response:**
```json
[
  {
    "id": "770e8400-e29b-41d4-a716-446655440002",
    "theme": "morning_insight",
    "name": "Daily Infrastructure Wisdom",
    "description": "Morning insights about protocols and infrastructure",
    "templates": [
      "{insight}\n\n{reminder}",
      "{insight}\n\n{tagline}"
    ],
    "insights": [
      "Protocols don't sleep. Neither should your infrastructure.",
      "The best agents are the ones you forget are running."
    ],
    "metadata": {},
    "active": true,
    "created_at": "2026-02-02T10:00:00Z",
    "updated_at": "2026-02-02T10:00:00Z"
  }
]
```

**Status Codes:**
- `200 OK` - Templates retrieved
- `500 Internal Server Error` - Database error

**Example:**
```bash
curl "http://localhost:3000/api/content/templates"
```

---

## Data Models

### ContentQueue

```typescript
{
  id: UUID,
  theme: string,
  text: string,
  tags: string[],
  image_prompt: string | null,
  image_path: string | null,
  status: "pending" | "approved" | "posted" | "failed" | "deleted",
  scheduled_at: DateTime | null,
  posted_at: DateTime | null,
  tweet_url: string | null,
  metadata: JSON,
  created_at: DateTime,
  updated_at: DateTime
}
```

### ContentMetrics

```typescript
{
  id: UUID,
  content_id: UUID,
  likes: number,
  retweets: number,
  replies: number,
  impressions: number,
  engagement_rate: number | null,
  fetched_at: DateTime,
  created_at: DateTime
}
```

### ContentTemplate

```typescript
{
  id: UUID,
  theme: string,
  name: string,
  description: string | null,
  templates: JSON,
  insights: JSON,
  metadata: JSON,
  active: boolean,
  created_at: DateTime,
  updated_at: DateTime
}
```

---

## Error Responses

All errors return JSON with the following structure:

```json
{
  "error": "Error message here"
}
```

**Common Status Codes:**
- `400 Bad Request` - Invalid request parameters
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server or database error
- `502 Bad Gateway` - Service unavailable (Erebus proxy only)

---

## Events

The service publishes events when configured with `EVENT_ENDPOINT`:

### content.generated

Published when new content is created.

```json
{
  "event_type": "content.generated",
  "content_id": "550e8400-e29b-41d4-a716-446655440000",
  "theme": "morning_insight",
  "status": "pending",
  "metadata": {},
  "timestamp": "2026-02-02T12:00:00Z"
}
```

### content.posted

Published when content is successfully posted to a platform.

```json
{
  "event_type": "content.posted",
  "content_id": "550e8400-e29b-41d4-a716-446655440000",
  "platform": "twitter",
  "url": "https://x.com/nullblock_io/status/123456",
  "timestamp": "2026-02-02T13:00:00Z"
}
```

### content.failed

Published when content generation or posting fails.

```json
{
  "event_type": "content.failed",
  "content_id": "550e8400-e29b-41d4-a716-446655440000",
  "error": "Error message",
  "timestamp": "2026-02-02T12:05:00Z"
}
```

---

## Rate Limiting

Currently no rate limiting. Future versions will implement per-user rate limits via Erebus.

---

## Changelog

### 0.1.0 (2026-02-02)

- Initial release
- 5 content themes
- Template system with JSON configuration
- Event publishing (HTTP-based)
- Erebus routing integration
- Queue management with approval workflow
- Metrics tracking

---

*For more information, see [SKILL.md](SKILL.md) or the main [NullBlock documentation](../../README.md).*
