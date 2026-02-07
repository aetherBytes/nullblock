use crate::config::tool_allowlist::is_tool_allowed_for_agent;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolAnnotations {
    #[serde(rename = "readOnlyHint", skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    #[serde(rename = "destructiveHint", skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
    #[serde(rename = "idempotentHint", skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,
}

impl McpToolAnnotations {
    pub fn read_only() -> Self {
        Self {
            read_only_hint: Some(true),
            destructive_hint: Some(false),
            idempotent_hint: Some(true),
        }
    }

    pub fn write() -> Self {
        Self {
            read_only_hint: Some(false),
            destructive_hint: Some(false),
            idempotent_hint: Some(false),
        }
    }

    pub fn destructive() -> Self {
        Self {
            read_only_hint: Some(false),
            destructive_hint: Some(true),
            idempotent_hint: Some(false),
        }
    }

    pub fn idempotent_write() -> Self {
        Self {
            read_only_hint: Some(false),
            destructive_hint: Some(false),
            idempotent_hint: Some(true),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<McpToolAnnotations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub content: Vec<McpContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl McpToolResult {
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            content: vec![McpContent {
                content_type: "text".to_string(),
                text: text.into(),
            }],
            is_error: None,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            content: vec![McpContent {
                content_type: "text".to_string(),
                text: text.into(),
            }],
            is_error: Some(true),
        }
    }
}

pub fn get_all_tools() -> Vec<McpTool> {
    let mut tools = Vec::new();
    tools.extend(get_engram_tools());
    tools.extend(get_profile_tools());
    tools.extend(get_hecate_tools());
    tools.extend(get_moros_tools());
    tools.extend(get_crossroads_tools());
    tools.extend(get_llm_tools());
    tools
}

pub fn get_crossroads_public_tools() -> Vec<McpTool> {
    get_crossroads_tools()
}

pub fn get_agent_tools() -> Vec<McpTool> {
    get_all_tools()
        .into_iter()
        .filter(|t| is_tool_allowed_for_agent(&t.name))
        .collect()
}

fn get_engram_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "engram_create".to_string(),
            description: "Create a new engram (memory unit) for a wallet. Engrams store persona, preference, strategy, knowledge, or compliance data.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address that owns this engram"
                    },
                    "engram_type": {
                        "type": "string",
                        "description": "Type of engram",
                        "enum": ["persona", "preference", "strategy", "knowledge", "compliance"]
                    },
                    "key": {
                        "type": "string",
                        "description": "Unique key for this engram (e.g., 'user.profile.base')"
                    },
                    "content": {
                        "type": "string",
                        "description": "JSON string content of the engram"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Tags for categorization and search"
                    },
                    "is_public": {
                        "type": "boolean",
                        "description": "Whether this engram is publicly visible"
                    }
                },
                "required": ["wallet_address", "engram_type", "key", "content"]
            }),
            annotations: Some(McpToolAnnotations::write()),
            tags: Some(vec!["hecate.engrams".to_string()]),
        },
        McpTool {
            name: "engram_get".to_string(),
            description: "Get a specific engram by wallet address and key.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address that owns the engram"
                    },
                    "key": {
                        "type": "string",
                        "description": "Engram key to look up"
                    }
                },
                "required": ["wallet_address", "key"]
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["hecate.engrams".to_string()]),
        },
        McpTool {
            name: "engram_search".to_string(),
            description: "Search engrams with filters by wallet, type, tags, and query.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Filter by wallet address"
                    },
                    "engram_type": {
                        "type": "string",
                        "description": "Filter by engram type",
                        "enum": ["persona", "preference", "strategy", "knowledge", "compliance"]
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Filter by tags"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum results to return (default 20)"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Offset for pagination"
                    }
                },
                "required": []
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["hecate.engrams".to_string()]),
        },
        McpTool {
            name: "engram_update".to_string(),
            description: "Update an existing engram's content and/or tags by its ID.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Engram ID to update"
                    },
                    "content": {
                        "type": "string",
                        "description": "New JSON string content"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "New tags (replaces existing)"
                    }
                },
                "required": ["id", "content"]
            }),
            annotations: Some(McpToolAnnotations::write()),
            tags: Some(vec!["hecate.engrams".to_string()]),
        },
        McpTool {
            name: "engram_delete".to_string(),
            description: "Delete an engram by its ID. This action is permanent.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Engram ID to delete"
                    }
                },
                "required": ["id"]
            }),
            annotations: Some(McpToolAnnotations::destructive()),
            tags: Some(vec!["hecate.engrams".to_string()]),
        },
        McpTool {
            name: "engram_list_by_type".to_string(),
            description: "List all engrams for a wallet filtered by engram type.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address to list engrams for"
                    },
                    "engram_type": {
                        "type": "string",
                        "description": "Type of engram to filter by",
                        "enum": ["persona", "preference", "strategy", "knowledge", "compliance"]
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum results (default 50)"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Offset for pagination"
                    }
                },
                "required": ["wallet_address", "engram_type"]
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["hecate.engrams".to_string()]),
        },
    ]
}

fn get_hecate_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "hecate_new_session".to_string(),
            description: "Create a new chat session. This clears the current conversation and starts fresh.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address of the user (required)"
                    }
                },
                "required": ["wallet_address"]
            }),
            annotations: Some(McpToolAnnotations::write()),
            tags: Some(vec!["hecate.session".to_string()]),
        },
        McpTool {
            name: "hecate_list_sessions".to_string(),
            description: "List all chat sessions for a wallet address. Returns session summaries sorted by most recently updated.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address of the user (required)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of sessions to return (default 20)"
                    }
                },
                "required": ["wallet_address"]
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["hecate.session".to_string()]),
        },
        McpTool {
            name: "hecate_resume_session".to_string(),
            description: "Resume an existing chat session. Loads the session's message history into the conversation.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address of the user (required)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "ID of the session to resume (required)"
                    }
                },
                "required": ["wallet_address", "session_id"]
            }),
            annotations: Some(McpToolAnnotations::write()),
            tags: Some(vec!["hecate.session".to_string()]),
        },
        McpTool {
            name: "hecate_delete_session".to_string(),
            description: "Delete a chat session. Cannot delete pinned sessions.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address of the user (required)"
                    },
                    "session_id": {
                        "type": "string",
                        "description": "ID of the session to delete (required)"
                    }
                },
                "required": ["wallet_address", "session_id"]
            }),
            annotations: Some(McpToolAnnotations::destructive()),
            tags: Some(vec!["hecate.session".to_string()]),
        },
        McpTool {
            name: "hecate_remember".to_string(),
            description: "Proactively save important context about a visitor. Use when they share preferences, facts, decisions, or anything worth remembering.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address of the visitor"
                    },
                    "key": {
                        "type": "string",
                        "description": "Dot-path key for the memory (e.g., 'visitor.preference.chains')"
                    },
                    "content": {
                        "type": "string",
                        "description": "The information to remember"
                    },
                    "engram_type": {
                        "type": "string",
                        "description": "Type of memory",
                        "enum": ["persona", "preference", "knowledge"]
                    }
                },
                "required": ["wallet_address", "key", "content"]
            }),
            annotations: Some(McpToolAnnotations::idempotent_write()),
            tags: Some(vec!["hecate.memory".to_string()]),
        },
        McpTool {
            name: "hecate_cleanup".to_string(),
            description: "Compact old conversation sessions. Keeps 5 most recent and all pinned sessions. Deletes the rest.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address to clean up sessions for"
                    }
                },
                "required": ["wallet_address"]
            }),
            annotations: Some(McpToolAnnotations::destructive()),
            tags: Some(vec!["hecate.management".to_string()]),
        },
        McpTool {
            name: "hecate_pin_engram".to_string(),
            description: "Pin an engram to protect it from cleanup/deletion. Pinned engrams cannot be deleted until unpinned.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Engram ID to pin"
                    }
                },
                "required": ["id"]
            }),
            annotations: Some(McpToolAnnotations::idempotent_write()),
            tags: Some(vec!["hecate.management".to_string()]),
        },
        McpTool {
            name: "hecate_set_model".to_string(),
            description: "Switch the AI model. Search by name or keyword (e.g., 'opus', 'claude-sonnet', 'gpt-4o', 'deepseek'). Returns best match and alternatives.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Model name or keyword to search for"
                    }
                },
                "required": ["query"]
            }),
            annotations: Some(McpToolAnnotations::write()),
            tags: Some(vec!["hecate.management".to_string()]),
        },
        McpTool {
            name: "hecate_unpin_engram".to_string(),
            description: "Remove pin protection from an engram, allowing it to be deleted or cleaned up.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Engram ID to unpin"
                    }
                },
                "required": ["id"]
            }),
            annotations: Some(McpToolAnnotations::write()),
            tags: Some(vec!["hecate.management".to_string()]),
        },
    ]
}

fn get_moros_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "moros_remember".to_string(),
            description: "Proactively save important context about a visitor. Use when they share preferences, facts, decisions, or anything worth remembering.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address of the visitor"
                    },
                    "key": {
                        "type": "string",
                        "description": "Dot-path key for the memory (e.g., 'visitor.preference.chains')"
                    },
                    "content": {
                        "type": "string",
                        "description": "The information to remember"
                    },
                    "engram_type": {
                        "type": "string",
                        "description": "Type of memory",
                        "enum": ["persona", "preference", "knowledge"]
                    }
                },
                "required": ["wallet_address", "key", "content"]
            }),
            annotations: Some(McpToolAnnotations::idempotent_write()),
            tags: Some(vec!["moros.memory".to_string()]),
        },
        McpTool {
            name: "moros_cleanup".to_string(),
            description: "Compact old conversation sessions. Keeps 5 most recent and all pinned sessions. Deletes the rest.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address to clean up sessions for"
                    }
                },
                "required": ["wallet_address"]
            }),
            annotations: Some(McpToolAnnotations::destructive()),
            tags: Some(vec!["moros.management".to_string()]),
        },
        McpTool {
            name: "moros_pin_engram".to_string(),
            description: "Pin an engram to protect it from cleanup/deletion. Pinned engrams cannot be deleted until unpinned.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Engram ID to pin"
                    }
                },
                "required": ["id"]
            }),
            annotations: Some(McpToolAnnotations::idempotent_write()),
            tags: Some(vec!["moros.management".to_string()]),
        },
        McpTool {
            name: "moros_unpin_engram".to_string(),
            description: "Remove pin protection from an engram, allowing it to be deleted or cleaned up.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Engram ID to unpin"
                    }
                },
                "required": ["id"]
            }),
            annotations: Some(McpToolAnnotations::write()),
            tags: Some(vec!["moros.management".to_string()]),
        },
        McpTool {
            name: "moros_set_model".to_string(),
            description: "Switch the AI model for Moros. Search by name or keyword (e.g., 'opus', 'claude-sonnet', 'gpt-4o', 'deepseek'). Returns best match and alternatives.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Model name or keyword to search for"
                    }
                },
                "required": ["query"]
            }),
            annotations: Some(McpToolAnnotations::write()),
            tags: Some(vec!["moros.management".to_string()]),
        },
    ]
}

fn get_llm_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "llm_chat".to_string(),
            description: "Generate a chat completion using the NullBlock LLM service. Routes to the best available model automatically.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "messages": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "role": { "type": "string", "enum": ["system", "user", "assistant"] },
                                "content": { "type": "string" }
                            },
                            "required": ["role", "content"]
                        },
                        "description": "Array of chat messages"
                    },
                    "model": {
                        "type": "string",
                        "description": "Model to use (optional, defaults to auto-routing)"
                    },
                    "max_tokens": {
                        "type": "integer",
                        "description": "Maximum tokens in response"
                    },
                    "temperature": {
                        "type": "number",
                        "description": "Sampling temperature (0.0-2.0)"
                    },
                    "system_prompt": {
                        "type": "string",
                        "description": "System prompt (alternative to including system message in messages array)"
                    }
                },
                "required": ["messages"]
            }),
            annotations: Some(McpToolAnnotations::write()),
            tags: Some(vec!["llm.service".to_string()]),
        },
        McpTool {
            name: "llm_list_models".to_string(),
            description: "List available LLM models. Optionally filter to free models only.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "free_only": {
                        "type": "boolean",
                        "description": "If true, only return free models (default: false)"
                    }
                },
                "required": []
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["llm.service".to_string()]),
        },
        McpTool {
            name: "llm_model_status".to_string(),
            description: "Get current LLM model routing info, provider health, and statistics.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["llm.service".to_string()]),
        },
        McpTool {
            name: "llm_set_model".to_string(),
            description: "Set the preferred LLM model for an agent. Search by name or keyword (e.g., 'opus', 'claude-sonnet', 'gpt-4o', 'deepseek', 'llama'). Any model available on OpenRouter can be used. Returns best match and alternatives.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_name": {
                        "type": "string",
                        "description": "Name of the agent to set model for (e.g., 'clawros', 'hecate', 'moros')"
                    },
                    "query": {
                        "type": "string",
                        "description": "Model name or keyword to search for"
                    }
                },
                "required": ["agent_name", "query"]
            }),
            annotations: Some(McpToolAnnotations::write()),
            tags: Some(vec!["llm.service".to_string()]),
        },
        McpTool {
            name: "llm_get_model".to_string(),
            description: "Get the current preferred LLM model for an agent.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_name": {
                        "type": "string",
                        "description": "Name of the agent to get model for (e.g., 'clawros', 'hecate', 'moros')"
                    }
                },
                "required": ["agent_name"]
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["llm.service".to_string()]),
        },
    ]
}

fn get_profile_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "user_profile_get".to_string(),
            description: "Get user profile engrams (persona type with user.profile.* keys) for a wallet.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address to get profile for"
                    }
                },
                "required": ["wallet_address"]
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["hecate.engrams".to_string(), "hecate.profile".to_string()]),
        },
        McpTool {
            name: "user_profile_update".to_string(),
            description: "Update a user profile engram field. Maps field name to engram key 'user.profile.<field>'.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "wallet_address": {
                        "type": "string",
                        "description": "Wallet address to update profile for"
                    },
                    "field": {
                        "type": "string",
                        "description": "Profile field name (maps to key suffix, e.g., 'base' -> 'user.profile.base')"
                    },
                    "content": {
                        "type": "string",
                        "description": "New JSON string content for the profile field"
                    }
                },
                "required": ["wallet_address", "field", "content"]
            }),
            annotations: Some(McpToolAnnotations::idempotent_write()),
            tags: Some(vec!["hecate.engrams".to_string(), "hecate.profile".to_string()]),
        },
    ]
}

fn get_crossroads_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "crossroads_list_tools".to_string(),
            description: "List all available tools in the Crossroads marketplace. Returns tools from all providers including ArbFarm, Agents, Protocols, and external MCP servers. No wallet required.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "description": "Optional filter by category (e.g., 'trading', 'memory', 'analysis')"
                    },
                    "provider": {
                        "type": "string",
                        "description": "Optional filter by provider (e.g., 'arbfarm', 'agents', 'protocols')"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of tools to return (default 20)"
                    }
                },
                "required": []
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["crossroads.discovery".to_string(), "public".to_string()]),
        },
        McpTool {
            name: "crossroads_get_tool_info".to_string(),
            description: "Get detailed information about a specific tool by name. Returns description, input schema, provider, and usage hints. No wallet required.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "tool_name": {
                        "type": "string",
                        "description": "Name of the tool to get info about"
                    }
                },
                "required": ["tool_name"]
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["crossroads.discovery".to_string(), "public".to_string()]),
        },
        McpTool {
            name: "crossroads_list_agents".to_string(),
            description: "List all available agents in the Crossroads marketplace. Returns agent names, descriptions, capabilities, and health status. No wallet required.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of agents to return (default 20)"
                    }
                },
                "required": []
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["crossroads.discovery".to_string(), "public".to_string()]),
        },
        McpTool {
            name: "crossroads_list_hot".to_string(),
            description: "List hot/trending items in the Crossroads marketplace. Returns featured tools and high-activity items. No wallet required.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["crossroads.discovery".to_string(), "public".to_string()]),
        },
        McpTool {
            name: "crossroads_get_stats".to_string(),
            description: "Get overall statistics about the Crossroads marketplace. Returns counts of tools, agents, protocols, and health status. No wallet required.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            annotations: Some(McpToolAnnotations::read_only()),
            tags: Some(vec!["crossroads.discovery".to_string(), "public".to_string()]),
        },
    ]
}
