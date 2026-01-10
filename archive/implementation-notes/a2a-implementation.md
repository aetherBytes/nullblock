# A2A Protocol Implementation Notes

Archived from CLAUDE.md - detailed implementation notes for A2A Protocol v0.3.0.

## Database Changes

**File**: `svc/nullblock-agents/migrations/001_create_tasks_table.sql`

- Added `context_id UUID` - Groups related tasks together
- Added `kind VARCHAR DEFAULT 'task'` - Always "task" per A2A spec
- Changed `status VARCHAR` to use A2A state values with CHECK constraint
- Added `status_message TEXT` - Optional human-readable status description
- Added `status_timestamp TIMESTAMPTZ` - When status was last updated
- Added `history JSONB DEFAULT '[]'` - Message array tracking conversation
- Added `artifacts JSONB DEFAULT '[]'` - Artifact array with task outputs

## Repository Methods

**File**: `svc/nullblock-agents/src/database/repositories/tasks.rs`

- `add_message_to_history(task_id, message)` - Appends message to history JSONB array using PostgreSQL || operator
- `add_artifact(task_id, artifact)` - Appends artifact to artifacts JSONB array
- `update_status_with_message(task_id, state, message)` - Updates status with optional message and timestamp

## Handler Integration

**File**: `svc/nullblock-agents/src/handlers/tasks.rs`

- Task creation adds initial user message to history with task description
- Message format: `{messageId, role: "user", parts: [{type: "text", text}], timestamp, taskId, contextId, kind: "message"}`

## Agent Integration

**File**: `svc/nullblock-agents/src/agents/hecate.rs:745-807`

- After task processing, adds agent response message to history with metadata (agent, model, processing_duration_ms)
- Creates completion artifact with result text and metadata (artifact_type: "completion_result", model, duration)
- Uses `update_status_with_message()` to set completion status with success message

## Protocols Service

**Files**: `svc/nullblock-protocols/`

- Added reqwest HTTP client dependency for service-to-service communication
- AppState contains http_client and agents_service_url (http://localhost:9003)
- Task handlers proxy to Agents service: GET /api/agents/tasks/:id, GET /api/agents/tasks, POST /api/agents/tasks/:id/cancel
- JSON-RPC handlers updated to pass AppState to task functions
- Response parsing extracts task data from `{"success": true, "data": {...}}` wrapper

## Frontend Types

**File**: `svc/hecate/src/types/tasks.ts`

- TaskStatus changed from string to object `{state: TaskState, message?: string, timestamp?: string}`
- TaskState type union with 9 A2A values
- A2AMessage interface with MessagePart union type (text | file | data)
- A2AArtifact interface with parts array
- Task interface updated with contextId, kind, status object, history, artifacts

## Bug Fixes

### Task Creation with User References
**File**: `svc/erebus/src/resources/agents/routes.rs:38-42`
- Fixed missing `network` field in `source_type` object causing task creation failures
- Error was: `"Failed to deserialize the JSON body into the target type: source_type: missing field 'network'"`
- Solution: Added `"network": wallet_chain` to default_source_type JSON

### Empty Response Handling
**File**: `svc/nullblock-agents/src/llm/factory.rs:161-164`
- Added validation to detect models returning 0 completion tokens
- Fallback chain now skips empty responses and tries next model
- Prevents silent failures with clear warning logs
