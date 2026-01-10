
# Authentication & Authorization Guide

## Overview

The NullBlock Protocols service implements a flexible authentication system that supports multiple auth mechanisms for both A2A and MCP protocols, following MCP's OAuth 2.1 recommendations for HTTP-based transports.

## MCP Authorization Specification Compliance

According to the [MCP Authorization Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/authorization):

- **HTTP-based transports SHOULD** conform to OAuth 2.1
- **STDIO transports SHOULD NOT** use this spec (credentials from environment)
- **Alternative transports MUST** follow established security best practices

Our implementation provides:
- ✅ OAuth 2.1-style Bearer token support
- ✅ Service-to-service authentication
- ✅ API key authentication
- ✅ Flexible auth requirements (optional or required)

## Authentication Mechanisms

### 1. Service-to-Service Authentication

**Purpose**: Secure communication between NullBlock services (Protocols ↔ Erebus ↔ Agents)

**Header**: `X-Service-Token`

**Configuration**:
```bash
# Set shared secret between services
export SERVICE_SECRET="your-secure-service-secret"

# Enable service auth requirement
export REQUIRE_SERVICE_AUTH=true  # In Erebus
```

**Usage**:
```bash
curl -X POST http://localhost:8001/mcp/jsonrpc \
  -H "X-Service-Token: your-secure-service-secret" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "initialize", "params": {...}, "id": 1}'
```

**When to use**: Internal service-to-service calls, automated systems

### 2. Bearer Token Authentication (OAuth 2.1 Style)

**Purpose**: External client authentication following MCP OAuth 2.1 recommendations

**Header**: `Authorization: Bearer <token>`

**Configuration**:
```bash
# Enable bearer token validation
export ENABLE_BEARER_TOKENS=true

# Require auth for MCP endpoints
export REQUIRE_MCP_AUTH=true
```

**Usage**:
```bash
curl -X POST http://localhost:8001/mcp/jsonrpc \
  -H "Authorization: Bearer your-access-token" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}'
```

**When to use**: MCP clients, external applications, OAuth 2.1 flows

### 3. API Key Authentication

**Purpose**: Simple authentication for known clients and integrations

**Headers**: 
- `X-API-Key: <key>` (preferred)
- `API-Key: <key>` (alternative)

**Configuration**:
```bash
# Comma-separated list of valid API keys
export API_KEYS="key1,key2,key3"

# Require auth globally
export REQUIRE_AUTH=true
```

**Usage**:
```bash
curl -X POST http://localhost:8001/a2a/jsonrpc \
  -H "X-API-Key: key1" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tasks/list", "params": {}, "id": 1}'
```

**When to use**: CI/CD pipelines, trusted integrations, development

## Environment Variables

### Global Auth Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `REQUIRE_AUTH` | `false` | Require authentication for all endpoints |
| `ENABLE_BEARER_TOKENS` | `true` | Enable OAuth 2.1-style bearer token validation |
| `API_KEYS` | `""` | Comma-separated list of valid API keys |
| `SERVICE_SECRET` | `nullblock-service-secret-dev` | Shared secret for service-to-service auth |

### Protocol-Specific Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `REQUIRE_A2A_AUTH` | `false` | Require authentication for A2A endpoints |
| `REQUIRE_MCP_AUTH` | `false` | Require authentication for MCP endpoints |

### Erebus Settings

| Variable | Default | Description |
|----------|---------|-------------|
| `REQUIRE_SERVICE_AUTH` | `false` | Require service token from protocols service |

## Authentication Flow

### External Client → Protocols Service

```
┌──────────────┐
│ MCP Client   │
└──────┬───────┘
       │ 1. Bearer Token
       ↓
┌──────────────┐
│ Protocols    │
│ Service      │
│              │
│ - Validates  │
│   Bearer     │
│   Token      │
│ - Validates  │
│   API Key    │
└──────────────┘
```

### Protocols Service → Erebus/Agents

```
┌──────────────┐
│ Protocols    │
│ Service      │
└──────┬───────┘
       │ 2. X-Service-Token
       ↓
┌──────────────┐
│ Erebus       │
│              │
│ - Validates  │
│   Service    │
│   Token      │
└──────┬───────┘
       │ 3. Forward request
       ↓
┌──────────────┐
│ Agents       │
│ Service      │
└──────────────┘
```

## Configuration Examples

### Development (No Auth)

```bash
# .env.development
REQUIRE_AUTH=false
REQUIRE_A2A_AUTH=false
REQUIRE_MCP_AUTH=false
REQUIRE_SERVICE_AUTH=false
```

### Production (Full Auth)

```bash
# .env.production
REQUIRE_AUTH=true
REQUIRE_A2A_AUTH=true
REQUIRE_MCP_AUTH=true
REQUIRE_SERVICE_AUTH=true

# Use strong secrets
SERVICE_SECRET="$(openssl rand -base64 32)"
API_KEYS="$(openssl rand -base64 32),$(openssl rand -base64 32)"

ENABLE_BEARER_TOKENS=true
```

### Hybrid (Service Auth Only)

```bash
# Service-to-service auth required
# External clients can still access without auth
REQUIRE_SERVICE_AUTH=true  # Erebus
SERVICE_SECRET="your-secret"

# External access open
REQUIRE_AUTH=false
REQUIRE_A2A_AUTH=false
REQUIRE_MCP_AUTH=false
```

## Security Best Practices

### 1. Use Strong Secrets

```bash
# Generate secure secrets
export SERVICE_SECRET="$(openssl rand -base64 32)"
export API_KEYS="$(openssl rand -base64 32)"
```

### 2. Enable TLS/HTTPS

```bash
# Use HTTPS in production
export PROTOCOLS_SERVICE_URL="https://protocols.nullblock.io"
export EREBUS_BASE_URL="https://erebus.nullblock.io"
```

### 3. Rotate Secrets Regularly

```bash
# Script to rotate secrets
#!/bin/bash
NEW_SECRET=$(openssl rand -base64 32)
echo "New SERVICE_SECRET: $NEW_SECRET"

# Update in all services
# Update in environment configs
```

### 4. Use Different Secrets Per Environment

```bash
# Development
SERVICE_SECRET="dev-secret"

# Staging
SERVICE_SECRET="$(vault read -field=value secret/staging/service-secret)"

# Production
SERVICE_SECRET="$(vault read -field=value secret/production/service-secret)"
```

### 5. Log Auth Events

All authentication attempts are logged:
- ✅ Successful auth
- ❌ Failed auth attempts
- ⚠️ Missing credentials

Monitor logs for suspicious activity.

## Implementation Details

### Auth Middleware Stack

#### A2A Protocol
```
Request → CORS → A2A Auth Middleware → Handler
                        ↓
                 - Service Token?
                 - API Key?
                 - Bearer Token?
```

#### MCP Protocol
```
Request → CORS → MCP Auth Middleware → Handler
                        ↓
                 - Service Token?
                 - API Key?
                 - Bearer Token?
```

### Auth Context

Authenticated requests have access to:
```rust
pub struct AuthContext {
    pub authenticated: bool,
    pub auth_type: Option<AuthType>,
    pub identity: Option<String>,
}

pub enum AuthType {
    ApiKey,
    BearerToken,
    ServiceToService,
}
```

### Validation Order

1. **Service Token** (highest priority)
2. **API Key**
3. **Bearer Token**
4. **No Auth** (if not required)

## Testing

### Test Service-to-Service Auth

```bash
# Protocols → Erebus
export SERVICE_SECRET="test-secret"

# Start Erebus with auth
cd svc/erebus
REQUIRE_SERVICE_AUTH=true SERVICE_SECRET="test-secret" cargo run

# Start Protocols
cd svc/nullblock-protocols
SERVICE_SECRET="test-secret" cargo run

# Test with service token
curl -X POST http://localhost:8001/mcp/jsonrpc \
  -H "X-Service-Token: test-secret" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "initialize", "params": {...}, "id": 1}'
```

### Test API Key Auth

```bash
# Start with API keys
API_KEYS="key1,key2" REQUIRE_MCP_AUTH=true cargo run

# Test valid key
curl -X POST http://localhost:8001/mcp/jsonrpc \
  -H "X-API-Key: key1" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "ping", "id": 1}'

# Test invalid key (should fail)
curl -X POST http://localhost:8001/mcp/jsonrpc \
  -H "X-API-Key: invalid" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "ping", "id": 1}'
```

### Test Bearer Token

```bash
# Start with bearer tokens enabled
ENABLE_BEARER_TOKENS=true REQUIRE_MCP_AUTH=true cargo run

# Test with bearer token
curl -X POST http://localhost:8001/mcp/jsonrpc \
  -H "Authorization: Bearer my-oauth-token" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "tools/list", "id": 1}'
```

## Troubleshooting

### "401 Unauthorized" Errors

1. **Check auth is configured**:
   ```bash
   echo $SERVICE_SECRET
   echo $API_KEYS
   ```

2. **Check auth requirements**:
   ```bash
   echo $REQUIRE_AUTH
   echo $REQUIRE_MCP_AUTH
   echo $REQUIRE_A2A_AUTH
   ```

3. **Check headers**:
   ```bash
   # Verify you're sending the right header
   curl -v http://localhost:8001/mcp/jsonrpc \
     -H "X-Service-Token: $SERVICE_SECRET"
   ```

4. **Check logs**:
   ```bash
   # Look for auth log messages
   tail -f logs/erebus.log | grep -E "✅|❌|⚠️"
   ```

### Service-to-Service Auth Not Working

1. **Verify same secret** in both services
2. **Check header name**: `X-Service-Token`
3. **Check secret format**: Plain string, no quotes in env

### API Keys Not Working

1. **Verify format**: Comma-separated, no spaces
2. **Check header**: `X-API-Key` or `API-Key`
3. **Verify not empty**: `echo $API_KEYS`

## Migration Guide

### Adding Auth to Existing Deployment

#### Phase 1: Optional Auth
```bash
# Add secrets but don't require
export SERVICE_SECRET="new-secret"
export API_KEYS="key1"
# Keep REQUIRE_* = false
```

#### Phase 2: Monitor
```bash
# Watch logs for auth usage
tail -f logs/*.log | grep "auth"
```

#### Phase 3: Enforce
```bash
# Enable requirements
export REQUIRE_SERVICE_AUTH=true
export REQUIRE_MCP_AUTH=true
export REQUIRE_A2A_AUTH=true
```

## OAuth 2.1 Future Enhancements

The current implementation provides Bearer token support compatible with OAuth 2.1. Future enhancements:

1. **Authorization Server Integration**
   - Token validation against OAuth server
   - Token introspection endpoint
   - Refresh token support

2. **Scope-based Authorization**
   - Define resource scopes
   - Validate token scopes
   - Fine-grained permissions

3. **PKCE Support**
   - Public client support
   - Code challenge/verifier

4. **Dynamic Client Registration**
   - Register MCP clients dynamically
   - Client credential management

## References

- [MCP Authorization Specification](https://modelcontextprotocol.io/specification/2025-11-25/authorization)
- [OAuth 2.1 Specification](https://oauth.net/2.1/)
- [Bearer Token RFC 6750](https://tools.ietf.org/html/rfc6750)
- [API Key Best Practices](https://cloud.google.com/endpoints/docs/openapi/when-why-api-key)


