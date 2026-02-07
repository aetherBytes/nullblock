use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};

use super::types::*;
use crate::server::AppState;

pub async fn initialize(
    State(_state): State<AppState>,
    request: InitializeRequest,
) -> Result<Json<InitializeResult>, StatusCode> {
    info!("🔌 MCP Initialize request received");
    info!(
        "  Client: {} v{}",
        request.client_info.name, request.client_info.version
    );
    info!("  Protocol Version: {}", request.protocol_version);

    if request.protocol_version != PROTOCOL_VERSION {
        warn!(
            "⚠️ Protocol version mismatch: client={}, server={}",
            request.protocol_version, PROTOCOL_VERSION
        );
    }

    let server_capabilities = ServerCapabilities {
        experimental: None,
        logging: None,
        prompts: Some(PromptsCapability {
            list_changed: false,
        }),
        resources: Some(ResourcesCapability {
            subscribe: false,
            list_changed: false,
        }),
        tools: Some(ToolsCapability {
            list_changed: false,
        }),
    };

    let result = InitializeResult {
        protocol_version: PROTOCOL_VERSION.to_string(),
        capabilities: server_capabilities,
        server_info: Implementation {
            name: "nullblock-protocols".to_string(),
            version: "0.1.0".to_string(),
            title: Some("NullBlock Protocols MCP Server".to_string()),
        },
        instructions: Some("NullBlock MCP server providing access to agent resources, tools for agent interaction and task management, and conversation prompts.".to_string()),
    };

    info!("✅ MCP initialization successful");

    Ok(Json(result))
}

pub async fn list_resources(
    State(state): State<AppState>,
) -> Result<Json<ListResourcesResult>, StatusCode> {
    info!("📋 MCP List Resources request");

    let agents_url = format!("{}/agents", state.agents_service_url);

    let resources = match state.http_client.get(&agents_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(agents) = json.get("data").and_then(|d| d.as_array()) {
                            agents
                                .iter()
                                .filter_map(|agent| {
                                    let name = agent.get("name")?.as_str()?;
                                    let status = agent.get("status")?.as_str()?;

                                    Some(Resource {
                                        uri: format!("agent://{}", name),
                                        name: name.to_string(),
                                        title: Some(format!("{} Agent", name)),
                                        description: Some(format!(
                                            "NullBlock Agent: {} ({})",
                                            name, status
                                        )),
                                        mime_type: Some("application/json".to_string()),
                                        annotations: None,
                                        size: None,
                                        meta: None,
                                    })
                                })
                                .collect()
                        } else {
                            vec![]
                        }
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to parse agents response: {}", e);
                        vec![]
                    }
                }
            } else {
                warn!("⚠️ Agents service returned status: {}", response.status());
                vec![]
            }
        }
        Err(e) => {
            warn!("⚠️ Failed to fetch agents: {}", e);
            vec![]
        }
    };

    info!("✅ Returning {} resources", resources.len());

    Ok(Json(ListResourcesResult { resources }))
}

pub async fn read_resource(
    State(state): State<AppState>,
    request: ReadResourceRequest,
) -> Result<Json<ReadResourceResult>, StatusCode> {
    info!("📖 MCP Read Resource: {}", request.uri);

    if !request.uri.starts_with("agent://") {
        warn!("⚠️ Unsupported resource URI scheme: {}", request.uri);
        return Err(StatusCode::BAD_REQUEST);
    }

    let agent_name = request.uri.strip_prefix("agent://").unwrap();
    let agent_url = format!("{}/agents/{}", state.agents_service_url, agent_name);

    match state.http_client.get(&agent_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        let contents = vec![ResourceContents {
                            uri: request.uri.clone(),
                            mime_type: Some("application/json".to_string()),
                            text: Some(text),
                            blob: None,
                        }];

                        info!("✅ Resource read successful");
                        Ok(Json(ReadResourceResult { contents }))
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to read response body: {}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            } else {
                warn!("⚠️ Agent service returned status: {}", response.status());
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            warn!("⚠️ Failed to fetch agent: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

pub async fn list_tools(
    State(state): State<AppState>,
) -> Result<Json<ListToolsResult>, StatusCode> {
    info!("🔧 MCP List Tools request");

    let mut tools = vec![
        Tool {
            name: "send_agent_message".to_string(),
            title: Some("Send Agent Message".to_string()),
            description: Some("Send a message to a NullBlock agent".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("agent_name".to_string(), json!({
                        "type": "string",
                        "description": "Name of the agent (e.g., 'hecate', 'siren')"
                    }));
                    props.insert("message".to_string(), json!({
                        "type": "string",
                        "description": "Message content to send to the agent"
                    }));
                    props
                }),
                required: Some(vec!["agent_name".to_string(), "message".to_string()]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
        Tool {
            name: "create_task".to_string(),
            title: Some("Create Task".to_string()),
            description: Some("Create a new task in the NullBlock task system".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("name".to_string(), json!({
                        "type": "string",
                        "description": "Task name"
                    }));
                    props.insert("description".to_string(), json!({
                        "type": "string",
                        "description": "Task description"
                    }));
                    props.insert("priority".to_string(), json!({
                        "type": "string",
                        "enum": ["low", "medium", "high", "critical"],
                        "description": "Task priority"
                    }));
                    props
                }),
                required: Some(vec!["name".to_string(), "description".to_string()]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
        Tool {
            name: "get_task_status".to_string(),
            title: Some("Get Task Status".to_string()),
            description: Some("Get the status of a task by ID".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("task_id".to_string(), json!({
                        "type": "string",
                        "description": "UUID of the task"
                    }));
                    props
                }),
                required: Some(vec!["task_id".to_string()]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
        // Engrams Tools - Context Keeping Protocol
        Tool {
            name: "list_engrams".to_string(),
            title: Some("List Engrams".to_string()),
            description: Some("List all engrams for a wallet address. Engrams are persistent context memories (personas, preferences, strategies, knowledge, compliance).".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("wallet_address".to_string(), json!({
                        "type": "string",
                        "description": "Wallet address to list engrams for"
                    }));
                    props.insert("engram_type".to_string(), json!({
                        "type": "string",
                        "enum": ["persona", "preference", "strategy", "knowledge", "compliance"],
                        "description": "Filter by engram type (optional)"
                    }));
                    props.insert("limit".to_string(), json!({
                        "type": "integer",
                        "description": "Maximum number of engrams to return (default: 50)"
                    }));
                    props
                }),
                required: Some(vec!["wallet_address".to_string()]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
        Tool {
            name: "get_engram".to_string(),
            title: Some("Get Engram".to_string()),
            description: Some("Get a specific engram by ID".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("engram_id".to_string(), json!({
                        "type": "string",
                        "description": "UUID of the engram"
                    }));
                    props
                }),
                required: Some(vec!["engram_id".to_string()]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
        Tool {
            name: "create_engram".to_string(),
            title: Some("Create Engram".to_string()),
            description: Some("Create a new engram (persistent context memory) for a wallet".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("wallet_address".to_string(), json!({
                        "type": "string",
                        "description": "Wallet address that owns the engram"
                    }));
                    props.insert("engram_type".to_string(), json!({
                        "type": "string",
                        "enum": ["persona", "preference", "strategy", "knowledge", "compliance"],
                        "description": "Type of engram"
                    }));
                    props.insert("key".to_string(), json!({
                        "type": "string",
                        "description": "Unique key for this engram within the wallet"
                    }));
                    props.insert("content".to_string(), json!({
                        "type": "string",
                        "description": "Content/value of the engram"
                    }));
                    props.insert("metadata".to_string(), json!({
                        "type": "object",
                        "description": "Optional metadata for the engram"
                    }));
                    props
                }),
                required: Some(vec![
                    "wallet_address".to_string(),
                    "engram_type".to_string(),
                    "key".to_string(),
                    "content".to_string()
                ]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
        Tool {
            name: "update_engram".to_string(),
            title: Some("Update Engram".to_string()),
            description: Some("Update an existing engram's content or metadata".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("engram_id".to_string(), json!({
                        "type": "string",
                        "description": "UUID of the engram to update"
                    }));
                    props.insert("content".to_string(), json!({
                        "type": "string",
                        "description": "New content for the engram (optional)"
                    }));
                    props.insert("metadata".to_string(), json!({
                        "type": "object",
                        "description": "New metadata for the engram (optional)"
                    }));
                    props
                }),
                required: Some(vec!["engram_id".to_string()]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
        Tool {
            name: "delete_engram".to_string(),
            title: Some("Delete Engram".to_string()),
            description: Some("Delete an engram by ID".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("engram_id".to_string(), json!({
                        "type": "string",
                        "description": "UUID of the engram to delete"
                    }));
                    props
                }),
                required: Some(vec!["engram_id".to_string()]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
        Tool {
            name: "search_engrams".to_string(),
            title: Some("Search Engrams".to_string()),
            description: Some("Search engrams by query string within a wallet".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("wallet_address".to_string(), json!({
                        "type": "string",
                        "description": "Wallet address to search within"
                    }));
                    props.insert("query".to_string(), json!({
                        "type": "string",
                        "description": "Search query"
                    }));
                    props.insert("engram_type".to_string(), json!({
                        "type": "string",
                        "enum": ["persona", "preference", "strategy", "knowledge", "compliance"],
                        "description": "Filter by engram type (optional)"
                    }));
                    props
                }),
                required: Some(vec!["wallet_address".to_string(), "query".to_string()]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
        Tool {
            name: "llm_set_model".to_string(),
            title: Some("Set LLM Model".to_string()),
            description: Some("Set the preferred LLM model for an agent. Search by name or keyword (e.g., 'opus', 'claude-sonnet', 'gpt-4o', 'deepseek', 'llama'). Any model available on OpenRouter can be used.".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("agent_name".to_string(), json!({
                        "type": "string",
                        "description": "Name of the agent to set model for (e.g., 'clawros', 'hecate', 'moros')"
                    }));
                    props.insert("query".to_string(), json!({
                        "type": "string",
                        "description": "Model name or keyword to search for"
                    }));
                    props
                }),
                required: Some(vec!["agent_name".to_string(), "query".to_string()]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
        Tool {
            name: "llm_get_model".to_string(),
            title: Some("Get LLM Model".to_string()),
            description: Some("Get the current preferred LLM model for an agent.".to_string()),
            input_schema: InputSchema {
                schema_type: "object".to_string(),
                properties: Some({
                    let mut props = HashMap::new();
                    props.insert("agent_name".to_string(), json!({
                        "type": "string",
                        "description": "Name of the agent to get model for (e.g., 'clawros', 'hecate', 'moros')"
                    }));
                    props
                }),
                required: Some(vec!["agent_name".to_string()]),
            },
            output_schema: None,
            annotations: None,
            meta: None,
        },
    ];

    // Fetch and aggregate ArbFarm tools with arb_ prefix
    let arbfarm_tools_url = format!("{}/mcp/manifest", state.arbfarm_url);
    match state.http_client.get(&arbfarm_tools_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(manifest) = response.json::<serde_json::Value>().await {
                    if let Some(arb_tools) = manifest.get("tools").and_then(|t| t.as_array()) {
                        for tool_value in arb_tools {
                            if let (Some(name), Some(description), Some(input_schema)) = (
                                tool_value.get("name").and_then(|n| n.as_str()),
                                tool_value.get("description").and_then(|d| d.as_str()),
                                tool_value.get("inputSchema"),
                            ) {
                                let prefixed_name = format!("arb_{}", name);
                                let mut props = HashMap::new();
                                if let Some(schema_props) =
                                    input_schema.get("properties").and_then(|p| p.as_object())
                                {
                                    for (key, val) in schema_props {
                                        props.insert(key.clone(), val.clone());
                                    }
                                }
                                let required: Vec<String> = input_schema
                                    .get("required")
                                    .and_then(|r| r.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect()
                                    })
                                    .unwrap_or_default();

                                tools.push(Tool {
                                    name: prefixed_name,
                                    title: Some(format!("ArbFarm: {}", name.replace('_', " "))),
                                    description: Some(format!("[ArbFarm] {}", description)),
                                    input_schema: InputSchema {
                                        schema_type: "object".to_string(),
                                        properties: if props.is_empty() {
                                            None
                                        } else {
                                            Some(props)
                                        },
                                        required: if required.is_empty() {
                                            None
                                        } else {
                                            Some(required)
                                        },
                                    },
                                    output_schema: None,
                                    annotations: None,
                                    meta: None,
                                });
                            }
                        }
                        info!("✅ Aggregated {} tools from ArbFarm", arb_tools.len());
                    }
                }
            } else {
                warn!("⚠️ ArbFarm service returned status: {}", response.status());
            }
        }
        Err(e) => {
            warn!(
                "⚠️ Failed to fetch ArbFarm tools: {}. ArbFarm tools will not be available.",
                e
            );
        }
    }

    info!("✅ Returning {} tools", tools.len());

    Ok(Json(ListToolsResult { tools }))
}

pub async fn call_tool(
    State(state): State<AppState>,
    request: CallToolRequest,
) -> Result<Json<CallToolResult>, StatusCode> {
    info!("🔨 MCP Call Tool: {}", request.name);

    // Route arb_* prefixed tools to ArbFarm service
    if request.name.starts_with("arb_") {
        let original_name = request.name.strip_prefix("arb_").unwrap();
        let arbfarm_call_url = format!("{}/mcp/call", state.arbfarm_url);

        let body = json!({
            "name": original_name,
            "arguments": request.arguments.clone().unwrap_or_default()
        });

        info!(
            "🔀 Routing tool '{}' to ArbFarm as '{}'",
            request.name, original_name
        );

        match state
            .http_client
            .post(&arbfarm_call_url)
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(result) => {
                            // Parse ArbFarm response format
                            let content = result
                                .get("content")
                                .and_then(|c| c.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|item| {
                                            let text = item.get("text").and_then(|t| t.as_str())?;
                                            Some(ContentBlock::Text {
                                                text: text.to_string(),
                                                annotations: None,
                                                meta: None,
                                            })
                                        })
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_else(|| {
                                    vec![ContentBlock::Text {
                                        text: result.to_string(),
                                        annotations: None,
                                        meta: None,
                                    }]
                                });

                            let is_error = result.get("isError").and_then(|e| e.as_bool());

                            return Ok(Json(CallToolResult {
                                content,
                                structured_content: None,
                                is_error,
                            }));
                        }
                        Err(e) => {
                            return Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text: format!("Failed to parse ArbFarm response: {}", e),
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(true),
                            }));
                        }
                    }
                } else {
                    return Ok(Json(CallToolResult {
                        content: vec![ContentBlock::Text {
                            text: format!("ArbFarm service error: {}", response.status()),
                            annotations: None,
                            meta: None,
                        }],
                        structured_content: None,
                        is_error: Some(true),
                    }));
                }
            }
            Err(e) => {
                return Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to connect to ArbFarm: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                }));
            }
        }
    }

    match request.name.as_str() {
        "send_agent_message" => {
            let args = request.arguments.unwrap_or_default();
            let agent_name = args
                .get("agent_name")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let message = args
                .get("message")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;

            let url = format!(
                "{}/api/agents/{}/chat",
                state.agents_service_url, agent_name
            );
            let body = json!({
                "message": message
            });

            match state.http_client.post(&url).json(&body).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text,
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(false),
                            })),
                            Err(e) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text: format!("Failed to read response: {}", e),
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(true),
                            })),
                        }
                    } else {
                        Ok(Json(CallToolResult {
                            content: vec![ContentBlock::Text {
                                text: format!("Agent returned status: {}", response.status()),
                                annotations: None,
                                meta: None,
                            }],
                            structured_content: None,
                            is_error: Some(true),
                        }))
                    }
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to send message: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        "create_task" => {
            let args = request.arguments.unwrap_or_default();
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let description = args
                .get("description")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let priority = args
                .get("priority")
                .and_then(|v| v.as_str())
                .unwrap_or("medium");

            let url = format!("{}/tasks", state.agents_service_url);
            let body = json!({
                "name": name,
                "description": description,
                "priority": priority,
                "task_type": "system",
                "category": "user_assigned"
            });

            match state.http_client.post(&url).json(&body).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text,
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(false),
                            })),
                            Err(e) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text: format!("Failed to read response: {}", e),
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(true),
                            })),
                        }
                    } else {
                        Ok(Json(CallToolResult {
                            content: vec![ContentBlock::Text {
                                text: format!(
                                    "Task service returned status: {}",
                                    response.status()
                                ),
                                annotations: None,
                                meta: None,
                            }],
                            structured_content: None,
                            is_error: Some(true),
                        }))
                    }
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to create task: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        "get_task_status" => {
            let args = request.arguments.unwrap_or_default();
            let task_id = args
                .get("task_id")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;

            let url = format!("{}/tasks/{}", state.agents_service_url, task_id);

            match state.http_client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text,
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(false),
                            })),
                            Err(e) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text: format!("Failed to read response: {}", e),
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(true),
                            })),
                        }
                    } else {
                        Ok(Json(CallToolResult {
                            content: vec![ContentBlock::Text {
                                text: format!(
                                    "Task not found or service error: {}",
                                    response.status()
                                ),
                                annotations: None,
                                meta: None,
                            }],
                            structured_content: None,
                            is_error: Some(true),
                        }))
                    }
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to fetch task: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        // Engrams Tools - All calls go through Erebus (dogfooding)
        "list_engrams" => {
            let args = request.arguments.unwrap_or_default();
            let wallet_address = args
                .get("wallet_address")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let engram_type = args.get("engram_type").and_then(|v| v.as_str());
            let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);

            let mut url = format!(
                "{}/api/engrams/wallet/{}",
                state.erebus_base_url, wallet_address
            );
            if let Some(etype) = engram_type {
                url = format!("{}?type={}&limit={}", url, etype, limit);
            } else {
                url = format!("{}?limit={}", url, limit);
            }

            info!("🧠 Listing engrams from Erebus: {}", url);

            match state.http_client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text,
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(false),
                            })),
                            Err(e) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text: format!("Failed to read response: {}", e),
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(true),
                            })),
                        }
                    } else {
                        Ok(Json(CallToolResult {
                            content: vec![ContentBlock::Text {
                                text: format!("Engrams service error: {}", response.status()),
                                annotations: None,
                                meta: None,
                            }],
                            structured_content: None,
                            is_error: Some(true),
                        }))
                    }
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to connect to Erebus: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        "get_engram" => {
            let args = request.arguments.unwrap_or_default();
            let engram_id = args
                .get("engram_id")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;

            let url = format!("{}/api/engrams/{}", state.erebus_base_url, engram_id);
            info!("🧠 Getting engram from Erebus: {}", url);

            match state.http_client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text,
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(false),
                            })),
                            Err(e) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text: format!("Failed to read response: {}", e),
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(true),
                            })),
                        }
                    } else {
                        Ok(Json(CallToolResult {
                            content: vec![ContentBlock::Text {
                                text: format!(
                                    "Engram not found or service error: {}",
                                    response.status()
                                ),
                                annotations: None,
                                meta: None,
                            }],
                            structured_content: None,
                            is_error: Some(true),
                        }))
                    }
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to connect to Erebus: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        "create_engram" => {
            let args = request.arguments.unwrap_or_default();
            let wallet_address = args
                .get("wallet_address")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let engram_type = args
                .get("engram_type")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let key = args
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let metadata = args.get("metadata").cloned();

            let url = format!("{}/api/engrams", state.erebus_base_url);
            let mut body = json!({
                "wallet_address": wallet_address,
                "engram_type": engram_type,
                "key": key,
                "content": content
            });
            if let Some(meta) = metadata {
                body.as_object_mut()
                    .unwrap()
                    .insert("metadata".to_string(), meta);
            }

            info!("🧠 Creating engram via Erebus: {}", url);

            match state.http_client.post(&url).json(&body).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text,
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(false),
                            })),
                            Err(e) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text: format!("Failed to read response: {}", e),
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(true),
                            })),
                        }
                    } else {
                        Ok(Json(CallToolResult {
                            content: vec![ContentBlock::Text {
                                text: format!("Failed to create engram: {}", response.status()),
                                annotations: None,
                                meta: None,
                            }],
                            structured_content: None,
                            is_error: Some(true),
                        }))
                    }
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to connect to Erebus: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        "update_engram" => {
            let args = request.arguments.unwrap_or_default();
            let engram_id = args
                .get("engram_id")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let content = args.get("content").and_then(|v| v.as_str());
            let metadata = args.get("metadata").cloned();

            let url = format!("{}/api/engrams/{}", state.erebus_base_url, engram_id);
            let mut body = json!({});
            if let Some(c) = content {
                body.as_object_mut()
                    .unwrap()
                    .insert("content".to_string(), json!(c));
            }
            if let Some(meta) = metadata {
                body.as_object_mut()
                    .unwrap()
                    .insert("metadata".to_string(), meta);
            }

            info!("🧠 Updating engram via Erebus: {}", url);

            match state.http_client.put(&url).json(&body).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text,
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(false),
                            })),
                            Err(e) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text: format!("Failed to read response: {}", e),
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(true),
                            })),
                        }
                    } else {
                        Ok(Json(CallToolResult {
                            content: vec![ContentBlock::Text {
                                text: format!("Failed to update engram: {}", response.status()),
                                annotations: None,
                                meta: None,
                            }],
                            structured_content: None,
                            is_error: Some(true),
                        }))
                    }
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to connect to Erebus: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        "delete_engram" => {
            let args = request.arguments.unwrap_or_default();
            let engram_id = args
                .get("engram_id")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;

            let url = format!("{}/api/engrams/{}", state.erebus_base_url, engram_id);
            info!("🧠 Deleting engram via Erebus: {}", url);

            match state.http_client.delete(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        Ok(Json(CallToolResult {
                            content: vec![ContentBlock::Text {
                                text: format!("Engram {} deleted successfully", engram_id),
                                annotations: None,
                                meta: None,
                            }],
                            structured_content: None,
                            is_error: Some(false),
                        }))
                    } else {
                        Ok(Json(CallToolResult {
                            content: vec![ContentBlock::Text {
                                text: format!("Failed to delete engram: {}", response.status()),
                                annotations: None,
                                meta: None,
                            }],
                            structured_content: None,
                            is_error: Some(true),
                        }))
                    }
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to connect to Erebus: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        "search_engrams" => {
            let args = request.arguments.unwrap_or_default();
            let wallet_address = args
                .get("wallet_address")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let engram_type = args.get("engram_type").and_then(|v| v.as_str());

            let url = format!("{}/api/engrams/search", state.erebus_base_url);
            let mut body = json!({
                "wallet_address": wallet_address,
                "query": query
            });
            if let Some(etype) = engram_type {
                body.as_object_mut()
                    .unwrap()
                    .insert("engram_type".to_string(), json!(etype));
            }

            info!("🧠 Searching engrams via Erebus: {}", url);

            match state.http_client.post(&url).json(&body).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(text) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text,
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(false),
                            })),
                            Err(e) => Ok(Json(CallToolResult {
                                content: vec![ContentBlock::Text {
                                    text: format!("Failed to read response: {}", e),
                                    annotations: None,
                                    meta: None,
                                }],
                                structured_content: None,
                                is_error: Some(true),
                            })),
                        }
                    } else {
                        Ok(Json(CallToolResult {
                            content: vec![ContentBlock::Text {
                                text: format!("Search failed: {}", response.status()),
                                annotations: None,
                                meta: None,
                            }],
                            structured_content: None,
                            is_error: Some(true),
                        }))
                    }
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to connect to Erebus: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        "llm_set_model" => {
            let args = request.arguments.unwrap_or_default();
            let agent_name = args.get("agent_name").and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;
            let query = args.get("query").and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;

            let url = format!("{}/v1/set-model", state.agents_service_url);
            let body = json!({
                "agent_name": agent_name,
                "query": query
            });

            match state.http_client.post(&url).json(&body).send().await {
                Ok(response) => {
                    let text = response.text().await.unwrap_or_else(|e| format!("Read error: {}", e));
                    Ok(Json(CallToolResult {
                        content: vec![ContentBlock::Text {
                            text,
                            annotations: None,
                            meta: None,
                        }],
                        structured_content: None,
                        is_error: None,
                    }))
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to connect to agents service: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        "llm_get_model" => {
            let args = request.arguments.unwrap_or_default();
            let agent_name = args.get("agent_name").and_then(|v| v.as_str())
                .ok_or(StatusCode::BAD_REQUEST)?;

            let url = format!("{}/v1/model-preference/{}", state.agents_service_url, agent_name);

            match state.http_client.get(&url).send().await {
                Ok(response) => {
                    let text = response.text().await.unwrap_or_else(|e| format!("Read error: {}", e));
                    Ok(Json(CallToolResult {
                        content: vec![ContentBlock::Text {
                            text,
                            annotations: None,
                            meta: None,
                        }],
                        structured_content: None,
                        is_error: None,
                    }))
                }
                Err(e) => Ok(Json(CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Failed to connect to agents service: {}", e),
                        annotations: None,
                        meta: None,
                    }],
                    structured_content: None,
                    is_error: Some(true),
                })),
            }
        }
        _ => {
            warn!("⚠️ Unknown tool: {}", request.name);
            Ok(Json(CallToolResult {
                content: vec![ContentBlock::Text {
                    text: format!("Unknown tool: {}", request.name),
                    annotations: None,
                    meta: None,
                }],
                structured_content: None,
                is_error: Some(true),
            }))
        }
    }
}

pub async fn list_prompts(
    State(_state): State<AppState>,
) -> Result<Json<ListPromptsResult>, StatusCode> {
    info!("💬 MCP List Prompts request");

    let prompts = vec![
        Prompt {
            name: "agent_chat".to_string(),
            title: Some("Agent Chat".to_string()),
            description: Some("Chat with a NullBlock agent".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "agent".to_string(),
                    title: Some("Agent Name".to_string()),
                    description: Some("Agent name (e.g., 'hecate', 'siren')".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "context".to_string(),
                    title: Some("Context".to_string()),
                    description: Some("Additional context for the conversation".to_string()),
                    required: Some(false),
                },
            ]),
            meta: None,
        },
        Prompt {
            name: "task_template".to_string(),
            title: Some("Task Template".to_string()),
            description: Some("Create a task from a template".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "type".to_string(),
                title: Some("Task Type".to_string()),
                description: Some(
                    "Task type (e.g., 'analysis', 'research', 'development')".to_string(),
                ),
                required: Some(true),
            }]),
            meta: None,
        },
    ];

    info!("✅ Returning {} prompts", prompts.len());

    Ok(Json(ListPromptsResult { prompts }))
}

pub async fn get_prompt(
    State(_state): State<AppState>,
    request: GetPromptRequest,
) -> Result<Json<GetPromptResult>, StatusCode> {
    info!("💬 MCP Get Prompt: {}", request.name);

    let args = request.arguments.unwrap_or_default();

    match request.name.as_str() {
        "agent_chat" => {
            let agent = args.get("agent").ok_or(StatusCode::BAD_REQUEST)?;
            let context = args.get("context").map(|s| s.as_str()).unwrap_or("");

            let mut messages = vec![PromptMessage {
                role: "system".to_string(),
                content: ContentBlock::Text {
                    text: format!("You are chatting with the NullBlock {} agent.", agent),
                    annotations: None,
                    meta: None,
                },
            }];

            if !context.is_empty() {
                messages.push(PromptMessage {
                    role: "system".to_string(),
                    content: ContentBlock::Text {
                        text: format!("Context: {}", context),
                        annotations: None,
                        meta: None,
                    },
                });
            }

            messages.push(PromptMessage {
                role: "user".to_string(),
                content: ContentBlock::Text {
                    text: "What would you like to discuss?".to_string(),
                    annotations: None,
                    meta: None,
                },
            });

            Ok(Json(GetPromptResult {
                description: Some(format!("Chat with the {} agent", agent)),
                messages,
            }))
        }
        "task_template" => {
            let task_type = args.get("type").ok_or(StatusCode::BAD_REQUEST)?;

            let messages = vec![
                PromptMessage {
                    role: "system".to_string(),
                    content: ContentBlock::Text {
                        text: format!("Creating a {} task template", task_type),
                        annotations: None,
                        meta: None,
                    },
                },
                PromptMessage {
                    role: "user".to_string(),
                    content: ContentBlock::Text {
                        text: format!("Please provide details for your {} task:", task_type),
                        annotations: None,
                        meta: None,
                    },
                },
            ];

            Ok(Json(GetPromptResult {
                description: Some(format!("Template for {} tasks", task_type)),
                messages,
            }))
        }
        _ => {
            warn!("⚠️ Unknown prompt: {}", request.name);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
