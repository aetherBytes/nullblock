# Official RMCP SDK Integration Guide

## Overview

The [official Anthropic Rust SDK for MCP](https://github.com/modelcontextprotocol/rust-sdk) (`rmcp` crate) is available as an optional dependency in this project. This guide explains when and how to use it.

## Current Status

- **SDK Version**: `rmcp` v0.8.0
- **Status**: Available but not currently used
- **Feature Flag**: `official-mcp-sdk`
- **Current Implementation**: Custom JSON-RPC 2.0 gateway

## Why We're Not Using It (Yet)

Our custom implementation is better suited for our use case because:

1. **Gateway Pattern**: We're building a protocol gateway/proxy, not a native MCP server
2. **HTTP/JSON-RPC First**: The SDK is designed for stdio/native transports, we need HTTP
3. **Service Integration**: We proxy to existing services (Agents, Tasks) rather than implementing MCP primitives directly
4. **Axum Integration**: Our custom handlers integrate seamlessly with our existing Axum router
5. **A2A Consistency**: Our MCP implementation mirrors our A2A architecture for consistency

## When to Consider Using the SDK

Consider migrating to the official SDK if:

1. **Native MCP Features Needed**: You want to use SDK-native features (sampling, stdio transport, etc.)
2. **Protocol Updates**: The SDK gets significant updates and we want automatic compatibility
3. **Type Safety**: You want stronger compile-time guarantees from the official types
4. **Reduced Maintenance**: You want to offload protocol implementation to the official SDK
5. **Alternative Transports**: You need WebSocket, stdio, or other transport mechanisms

## How to Enable the SDK

### 1. Build with the Feature Flag

```bash
cargo build --features official-mcp-sdk
```

### 2. Import in Code

```rust
#[cfg(feature = "official-mcp-sdk")]
use rmcp::{Server, ServerCapabilities, Implementation};

#[cfg(feature = "official-mcp-sdk")]
use rmcp::prelude::*;
```

### 3. Example SDK Usage

Here's how you would use the official SDK to create an MCP server:

```rust
use rmcp::{Server, ServerBuilder, Implementation};
use rmcp::protocol::{ServerCapabilities, ToolsCapability};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server with capabilities
    let server = ServerBuilder::new()
        .with_server_info(Implementation {
            name: "nullblock-mcp".to_string(),
            version: "0.1.0".to_string(),
        })
        .with_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: false,
            }),
            resources: Some(ResourcesCapability {
                subscribe: false,
                list_changed: false,
            }),
            prompts: Some(PromptsCapability {
                list_changed: false,
            }),
            ..Default::default()
        })
        .build();

    // Register tools, resources, prompts
    // (SDK provides handler traits to implement)

    // Start server
    server.run().await?;

    Ok(())
}
```

## Migration Path

If we decide to migrate to the official SDK:

### Phase 1: Parallel Implementation
1. Keep custom implementation at `/mcp` endpoint
2. Add SDK-based implementation at `/mcp-sdk` endpoint
3. Run both in parallel for testing

### Phase 2: Feature Parity
1. Implement all our tools using SDK handlers
2. Implement resources using SDK resource providers
3. Implement prompts using SDK prompt providers
4. Add HTTP transport adapter for SDK

### Phase 3: Cutover
1. Switch default endpoint to SDK implementation
2. Deprecate custom implementation
3. Remove custom types/handlers after migration period

## SDK Architecture

The `rmcp` SDK provides:

### Core Types
- `Server` - Main server struct
- `ServerBuilder` - Builder pattern for server setup
- `ServerCapabilities` - Capability negotiation
- `Implementation` - Server/client info

### Handler Traits
- `ToolHandler` - Implement for custom tools
- `ResourceProvider` - Implement for resources
- `PromptProvider` - Implement for prompts

### Transports
- **Stdio** - Standard input/output (default)
- **Custom** - Implement `Transport` trait for HTTP, WebSocket, etc.

### Macros
The `rmcp-macros` crate provides procedural macros:

```rust
use rmcp_macros::tool;

#[tool]
async fn send_message(
    #[arg(description = "Agent name")] agent: String,
    #[arg(description = "Message content")] message: String,
) -> Result<String, Error> {
    // Implementation
}
```

## Current vs. SDK Comparison

| Aspect | Current Implementation | Official SDK |
|--------|----------------------|--------------|
| Transport | HTTP/JSON-RPC | Stdio (HTTP custom) |
| Integration | Direct Axum handlers | Separate server |
| Types | Custom Serde types | SDK types |
| Validation | Manual | Built-in |
| Protocol Updates | Manual updates | Automatic |
| Service Proxying | Native | Requires adapter |
| Maintenance | Self-maintained | Anthropic-maintained |

## Resources

- **SDK Repository**: https://github.com/modelcontextprotocol/rust-sdk
- **Crate Documentation**: https://docs.rs/rmcp/latest/rmcp/
- **Examples**: https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples
- **MCP Specification**: https://modelcontextprotocol.io/specification/2025-11-25/basic

## Decision Log

### 2025-10-05: Initial Implementation
- **Decision**: Use custom JSON-RPC gateway implementation
- **Rationale**: Better fit for our protocol gateway pattern
- **Action**: Add SDK as optional dependency for future use

### Future: TBD
- Monitor SDK development for HTTP transport improvements
- Re-evaluate when SDK adds gateway/proxy patterns
- Consider migration if protocol complexity increases

## Contributing

When considering SDK integration:

1. Test with the feature flag enabled: `cargo test --features official-mcp-sdk`
2. Ensure parallel implementations don't conflict
3. Document any SDK-specific behavior
4. Keep custom implementation as fallback during migration
5. Update this guide with learnings

## Questions?

For questions about:
- **Current Implementation**: See `MCP_IMPLEMENTATION.md`
- **SDK Usage**: See [official SDK docs](https://github.com/modelcontextprotocol/rust-sdk)
- **Migration**: Open an issue or discussion

