use serde_json::Value;
use tracing::{error, info};

use super::tools::McpToolResult;
use crate::engrams::{CreateEngramRequest, EngramsClient, SearchRequest};
use crate::models::LLMRequest;
use crate::server::AppState;
use std::sync::Arc;

pub async fn execute_tool_with_engrams(
    engrams_client: &Arc<EngramsClient>,
    name: &str,
    args: Value,
) -> McpToolResult {
    execute_tool_agent(engrams_client, name, args).await
}

pub async fn execute_tool(
    state: &AppState,
    name: &str,
    args: Value,
) -> McpToolResult {
    info!("Executing MCP tool: {}", name);

    let engrams_client = &state.engrams_client;

    match name {
        "engram_create" => handle_engram_create(engrams_client, args).await,
        "engram_get" => handle_engram_get(engrams_client, args).await,
        "engram_search" => handle_engram_search(engrams_client, args).await,
        "engram_update" => handle_engram_update(engrams_client, args).await,
        "engram_delete" => handle_engram_delete(engrams_client, args).await,
        "engram_list_by_type" => handle_engram_list_by_type(engrams_client, args).await,
        "user_profile_get" => handle_user_profile_get(engrams_client, args).await,
        "user_profile_update" => handle_user_profile_update(engrams_client, args).await,
        "hecate_remember" => handle_hecate_remember(engrams_client, args).await,
        "hecate_cleanup" => handle_hecate_cleanup(engrams_client, args).await,
        "hecate_pin_engram" => handle_hecate_pin(engrams_client, args).await,
        "hecate_unpin_engram" => handle_hecate_unpin(engrams_client, args).await,
        "hecate_new_session" => handle_hecate_new_session(engrams_client, args).await,
        "hecate_list_sessions" => handle_hecate_list_sessions(engrams_client, args).await,
        "hecate_resume_session" => handle_hecate_resume_session(engrams_client, args).await,
        "hecate_delete_session" => handle_hecate_delete_session(engrams_client, args).await,
        "moros_remember" => handle_moros_remember(engrams_client, args).await,
        "moros_cleanup" => handle_hecate_cleanup(engrams_client, args).await,
        "moros_pin_engram" => handle_hecate_pin(engrams_client, args).await,
        "moros_unpin_engram" => handle_hecate_unpin(engrams_client, args).await,
        // Crossroads discovery tools (public, no wallet required)
        "crossroads_list_tools" => handle_crossroads_list_tools(args).await,
        "crossroads_get_tool_info" => handle_crossroads_get_tool_info(args).await,
        "crossroads_list_agents" => handle_crossroads_list_agents(args).await,
        "crossroads_list_hot" => handle_crossroads_list_hot(args).await,
        "crossroads_get_stats" => handle_crossroads_get_stats(args).await,
        "llm_chat" => handle_llm_chat(state, args).await,
        "llm_list_models" => handle_llm_list_models(state, args).await,
        "llm_model_status" => handle_llm_model_status(state).await,
        "llm_set_model" => handle_llm_set_model(state, args).await,
        "llm_get_model" => handle_llm_get_model(state, args).await,
        _ => McpToolResult::error(format!("Unknown tool: {}", name)),
    }
}

async fn handle_engram_create(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };
    let engram_type = match args.get("engram_type").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return McpToolResult::error("Missing required field: engram_type"),
    };
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return McpToolResult::error("Missing required field: key"),
    };
    let content = match args.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return McpToolResult::error("Missing required field: content"),
    };
    let tags = args.get("tags").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    });
    let is_public = args.get("is_public").and_then(|v| v.as_bool());

    let request = CreateEngramRequest {
        wallet_address,
        engram_type,
        key,
        content,
        metadata: None,
        tags,
        is_public,
    };

    match client.create_engram(request).await {
        Ok(engram) => match serde_json::to_string_pretty(&engram) {
            Ok(json) => McpToolResult::success(json),
            Err(e) => McpToolResult::error(format!("Failed to serialize engram: {}", e)),
        },
        Err(e) => McpToolResult::error(format!("Failed to create engram: {}", e)),
    }
}

async fn handle_engram_get(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w,
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return McpToolResult::error("Missing required field: key"),
    };

    match client.get_engram_by_wallet_key(wallet_address, key).await {
        Ok(Some(engram)) => match serde_json::to_string_pretty(&engram) {
            Ok(json) => McpToolResult::success(json),
            Err(e) => McpToolResult::error(format!("Failed to serialize engram: {}", e)),
        },
        Ok(None) => McpToolResult::success(
            serde_json::json!({"found": false, "message": "Engram not found"}).to_string(),
        ),
        Err(e) => McpToolResult::error(format!("Failed to get engram: {}", e)),
    }
}

async fn handle_engram_search(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = args
        .get("wallet_address")
        .and_then(|v| v.as_str())
        .map(String::from);
    let engram_type = args
        .get("engram_type")
        .and_then(|v| v.as_str())
        .map(String::from);
    let tags = args.get("tags").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    });
    let limit = args.get("limit").and_then(|v| v.as_i64()).or(Some(20));
    let offset = args.get("offset").and_then(|v| v.as_i64());

    let request = SearchRequest {
        wallet_address,
        engram_type,
        query: None,
        tags,
        limit,
        offset,
    };

    match client.search_engrams(request).await {
        Ok(engrams) => {
            let result = serde_json::json!({
                "count": engrams.len(),
                "engrams": engrams,
            });
            McpToolResult::success(result.to_string())
        }
        Err(e) => McpToolResult::error(format!("Failed to search engrams: {}", e)),
    }
}

async fn handle_engram_update(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(i) => i,
        None => return McpToolResult::error("Missing required field: id"),
    };
    let content = match args.get("content").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return McpToolResult::error("Missing required field: content"),
    };
    let tags = args.get("tags").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    });

    match client.update_engram(id, content, tags).await {
        Ok(engram) => match serde_json::to_string_pretty(&engram) {
            Ok(json) => McpToolResult::success(json),
            Err(e) => McpToolResult::error(format!("Failed to serialize engram: {}", e)),
        },
        Err(e) => McpToolResult::error(format!("Failed to update engram: {}", e)),
    }
}

async fn handle_engram_delete(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(i) => i,
        None => return McpToolResult::error("Missing required field: id"),
    };

    if let Ok(Some(engram)) = client.get_engram_by_id(id).await {
        if engram.tags.contains(&"pinned".to_string()) {
            return McpToolResult::error("Cannot delete pinned engram. Remove 'pinned' tag first.");
        }
    }

    match client.delete_engram(id).await {
        Ok(()) => {
            McpToolResult::success(serde_json::json!({"deleted": true, "id": id}).to_string())
        }
        Err(e) => McpToolResult::error(format!("Failed to delete engram: {}", e)),
    }
}

async fn handle_engram_list_by_type(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };
    let engram_type = match args.get("engram_type").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return McpToolResult::error("Missing required field: engram_type"),
    };
    let limit = args.get("limit").and_then(|v| v.as_i64()).or(Some(50));
    let offset = args.get("offset").and_then(|v| v.as_i64());

    let request = SearchRequest {
        wallet_address: Some(wallet_address),
        engram_type: Some(engram_type),
        query: None,
        tags: None,
        limit,
        offset,
    };

    match client.search_engrams(request).await {
        Ok(engrams) => {
            let result = serde_json::json!({
                "count": engrams.len(),
                "engrams": engrams,
            });
            McpToolResult::success(result.to_string())
        }
        Err(e) => McpToolResult::error(format!("Failed to list engrams: {}", e)),
    }
}

async fn handle_user_profile_get(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };

    let request = SearchRequest {
        wallet_address: Some(wallet_address),
        engram_type: Some("persona".to_string()),
        query: None,
        tags: Some(vec!["profile".to_string()]),
        limit: Some(20),
        offset: None,
    };

    match client.search_engrams(request).await {
        Ok(engrams) => {
            let profile_engrams: Vec<_> = engrams
                .into_iter()
                .filter(|e| {
                    e.key.starts_with("user.profile.") || e.key.starts_with("user.preferences.")
                })
                .collect();

            let result = serde_json::json!({
                "count": profile_engrams.len(),
                "profile": profile_engrams,
            });
            McpToolResult::success(result.to_string())
        }
        Err(e) => McpToolResult::error(format!("Failed to get user profile: {}", e)),
    }
}

async fn handle_user_profile_update(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };
    let field = match args.get("field").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return McpToolResult::error("Missing required field: field"),
    };
    let content = match args.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return McpToolResult::error("Missing required field: content"),
    };

    let key = format!("user.profile.{}", field);

    let request = CreateEngramRequest {
        wallet_address,
        engram_type: "persona".to_string(),
        key,
        content,
        metadata: None,
        tags: Some(vec!["user".to_string(), "profile".to_string()]),
        is_public: Some(false),
    };

    match client.upsert_engram(request).await {
        Ok(engram) => match serde_json::to_string_pretty(&engram) {
            Ok(json) => McpToolResult::success(json),
            Err(e) => McpToolResult::error(format!("Failed to serialize engram: {}", e)),
        },
        Err(e) => McpToolResult::error(format!("Failed to update profile: {}", e)),
    }
}

async fn handle_moros_remember(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return McpToolResult::error("Missing required field: key"),
    };
    let content = match args.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return McpToolResult::error("Missing required field: content"),
    };
    let engram_type = args
        .get("engram_type")
        .and_then(|v| v.as_str())
        .unwrap_or("knowledge")
        .to_string();

    let request = CreateEngramRequest {
        wallet_address,
        engram_type,
        key,
        content,
        metadata: None,
        tags: Some(vec!["moros".to_string(), "auto".to_string()]),
        is_public: Some(false),
    };

    match client.upsert_engram(request).await {
        Ok(engram) => McpToolResult::success(
            serde_json::json!({"remembered": true, "key": engram.key, "id": engram.id}).to_string(),
        ),
        Err(e) => McpToolResult::error(format!("Failed to remember: {}", e)),
    }
}

async fn handle_hecate_remember(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return McpToolResult::error("Missing required field: key"),
    };
    let content = match args.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return McpToolResult::error("Missing required field: content"),
    };
    let engram_type = args
        .get("engram_type")
        .and_then(|v| v.as_str())
        .unwrap_or("knowledge")
        .to_string();

    let request = CreateEngramRequest {
        wallet_address,
        engram_type,
        key,
        content,
        metadata: None,
        tags: Some(vec!["hecate".to_string(), "auto".to_string()]),
        is_public: Some(false),
    };

    match client.upsert_engram(request).await {
        Ok(engram) => McpToolResult::success(
            serde_json::json!({"remembered": true, "key": engram.key, "id": engram.id}).to_string(),
        ),
        Err(e) => McpToolResult::error(format!("Failed to remember: {}", e)),
    }
}

async fn handle_hecate_cleanup(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };

    let request = SearchRequest {
        wallet_address: Some(wallet_address),
        engram_type: Some("conversation".to_string()),
        query: None,
        tags: None,
        limit: Some(100),
        offset: None,
    };

    let engrams = match client.search_engrams(request).await {
        Ok(e) => e,
        Err(e) => return McpToolResult::error(format!("Failed to search sessions: {}", e)),
    };

    let pinned_count = engrams
        .iter()
        .filter(|e| e.tags.contains(&"pinned".to_string()))
        .count();
    let mut deletable: Vec<_> = engrams
        .into_iter()
        .filter(|e| !e.tags.contains(&"pinned".to_string()))
        .collect();

    deletable.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    let to_delete = if deletable.len() > 5 {
        &deletable[5..]
    } else {
        &[]
    };

    let mut deleted = 0;
    for engram in to_delete {
        if client.delete_engram(&engram.id).await.is_ok() {
            deleted += 1;
        }
    }

    let retained = deletable.len().saturating_sub(deleted);
    McpToolResult::success(
        serde_json::json!({
            "deleted": deleted,
            "retained": retained,
            "pinned": pinned_count,
            "message": format!("{} sessions removed, {} retained, {} pinned (protected)", deleted, retained, pinned_count)
        }).to_string()
    )
}

async fn handle_hecate_pin(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(i) => i,
        None => return McpToolResult::error("Missing required field: id"),
    };

    let engram = match client.get_engram_by_id(id).await {
        Ok(Some(e)) => e,
        Ok(None) => return McpToolResult::error("Engram not found"),
        Err(e) => return McpToolResult::error(format!("Failed to fetch engram: {}", e)),
    };

    let mut tags = engram.tags.clone();
    if !tags.contains(&"pinned".to_string()) {
        tags.push("pinned".to_string());
    }

    match client.update_engram(id, &engram.content, Some(tags)).await {
        Ok(_) => McpToolResult::success(serde_json::json!({"pinned": true, "id": id}).to_string()),
        Err(e) => McpToolResult::error(format!("Failed to pin engram: {}", e)),
    }
}

async fn handle_hecate_unpin(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(i) => i,
        None => return McpToolResult::error("Missing required field: id"),
    };

    let engram = match client.get_engram_by_id(id).await {
        Ok(Some(e)) => e,
        Ok(None) => return McpToolResult::error("Engram not found"),
        Err(e) => return McpToolResult::error(format!("Failed to fetch engram: {}", e)),
    };

    let tags: Vec<String> = engram.tags.into_iter().filter(|t| t != "pinned").collect();

    match client.update_engram(id, &engram.content, Some(tags)).await {
        Ok(_) => {
            McpToolResult::success(serde_json::json!({"unpinned": true, "id": id}).to_string())
        }
        Err(e) => McpToolResult::error(format!("Failed to unpin engram: {}", e)),
    }
}

// ==================== Session Management Handlers ====================

async fn handle_hecate_new_session(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };

    let session_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let session_content = serde_json::json!({
        "session_id": session_id,
        "title": "New conversation",
        "message_count": 0,
        "messages": [],
        "created_at": now,
        "updated_at": now
    });

    let key = format!("chat.session.{}", session_id);

    let request = CreateEngramRequest {
        wallet_address: wallet_address.clone(),
        engram_type: "conversation".to_string(),
        key: key.clone(),
        content: session_content.to_string(),
        metadata: None,
        tags: Some(vec![
            "chat".to_string(),
            "session".to_string(),
            "hecate".to_string(),
        ]),
        is_public: Some(false),
    };

    match client.create_engram(request).await {
        Ok(engram) => {
            let result = serde_json::json!({
                "success": true,
                "session_id": session_id,
                "engram_id": engram.id,
                "title": "New conversation",
                "message_count": 0,
                "created_at": now,
                "updated_at": now
            });
            McpToolResult::success(result.to_string())
        }
        Err(e) => McpToolResult::error(format!("Failed to create session: {}", e)),
    }
}

async fn handle_hecate_list_sessions(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };

    let limit = args.get("limit").and_then(|v| v.as_i64()).or(Some(20));

    let request = SearchRequest {
        wallet_address: Some(wallet_address.clone()),
        engram_type: Some("conversation".to_string()),
        query: None,
        tags: Some(vec!["session".to_string(), "hecate".to_string()]),
        limit,
        offset: None,
    };

    match client.search_engrams(request).await {
        Ok(engrams) => {
            let mut sessions: Vec<serde_json::Value> = engrams
                .into_iter()
                .filter(|e| e.wallet_address == wallet_address)
                .filter_map(|engram| {
                    let content: serde_json::Value = serde_json::from_str(&engram.content).ok()?;
                    Some(serde_json::json!({
                        "session_id": content.get("session_id")?.as_str()?,
                        "engram_id": engram.id,
                        "title": content.get("title")?.as_str().unwrap_or("Untitled"),
                        "message_count": content.get("message_count")?.as_u64().unwrap_or(0),
                        "created_at": content.get("created_at")?.as_str()?,
                        "updated_at": content.get("updated_at")?.as_str()?,
                        "is_pinned": engram.tags.contains(&"pinned".to_string())
                    }))
                })
                .collect();

            sessions.sort_by(|a, b| {
                let a_time = a.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
                let b_time = b.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
                b_time.cmp(a_time)
            });

            let result = serde_json::json!({
                "success": true,
                "sessions": sessions,
                "count": sessions.len()
            });
            McpToolResult::success(result.to_string())
        }
        Err(e) => McpToolResult::error(format!("Failed to list sessions: {}", e)),
    }
}

async fn handle_hecate_resume_session(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w,
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };
    let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return McpToolResult::error("Missing required field: session_id"),
    };

    let key = format!("chat.session.{}", session_id);

    match client.get_engram_by_wallet_key(wallet_address, &key).await {
        Ok(Some(engram)) => {
            if engram.wallet_address != wallet_address {
                return McpToolResult::error("Cannot access session from another wallet");
            }

            let content: serde_json::Value = match serde_json::from_str(&engram.content) {
                Ok(c) => c,
                Err(e) => return McpToolResult::error(format!("Failed to parse session: {}", e)),
            };

            let result = serde_json::json!({
                "success": true,
                "session": content,
                "engram_id": engram.id
            });
            McpToolResult::success(result.to_string())
        }
        Ok(None) => McpToolResult::error(format!("Session {} not found", session_id)),
        Err(e) => McpToolResult::error(format!("Failed to fetch session: {}", e)),
    }
}

async fn handle_hecate_delete_session(client: &Arc<EngramsClient>, args: Value) -> McpToolResult {
    let wallet_address = match args.get("wallet_address").and_then(|v| v.as_str()) {
        Some(w) => w,
        None => return McpToolResult::error("Missing required field: wallet_address"),
    };
    let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return McpToolResult::error("Missing required field: session_id"),
    };

    let key = format!("chat.session.{}", session_id);

    match client.get_engram_by_wallet_key(wallet_address, &key).await {
        Ok(Some(engram)) => {
            if engram.wallet_address != wallet_address {
                return McpToolResult::error("Cannot delete session from another wallet");
            }

            if engram.tags.contains(&"pinned".to_string()) {
                return McpToolResult::error("Cannot delete pinned session. Unpin first.");
            }

            match client.delete_engram(&engram.id).await {
                Ok(()) => {
                    let result = serde_json::json!({
                        "success": true,
                        "deleted": true,
                        "session_id": session_id
                    });
                    McpToolResult::success(result.to_string())
                }
                Err(e) => McpToolResult::error(format!("Failed to delete session: {}", e)),
            }
        }
        Ok(None) => McpToolResult::error(format!("Session {} not found", session_id)),
        Err(e) => McpToolResult::error(format!("Failed to fetch session: {}", e)),
    }
}

// Crossroads Discovery Handlers (public, no wallet required)

fn get_erebus_base_url() -> String {
    std::env::var("EREBUS_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

async fn handle_crossroads_list_tools(args: Value) -> McpToolResult {
    let erebus_url = get_erebus_base_url();
    let url = format!("{}/api/discovery/tools", erebus_url);

    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let category_filter = args.get("category").and_then(|v| v.as_str());
                        let provider_filter = args.get("provider").and_then(|v| v.as_str());
                        let limit =
                            args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

                        let tools = data.get("tools").and_then(|t| t.as_array());
                        let filtered: Vec<serde_json::Value> = match tools {
                            Some(t) => t
                                .iter()
                                .filter(|tool| {
                                    let cat_ok = category_filter.map_or(true, |cat| {
                                        tool.get("category")
                                            .and_then(|c| c.as_str())
                                            .map_or(false, |c| {
                                                c.to_lowercase().contains(&cat.to_lowercase())
                                            })
                                    });
                                    let prov_ok = provider_filter.map_or(true, |prov| {
                                        tool.get("provider")
                                            .and_then(|p| p.as_str())
                                            .map_or(false, |p| {
                                                p.to_lowercase().contains(&prov.to_lowercase())
                                            })
                                    });
                                    cat_ok && prov_ok
                                })
                                .take(limit)
                                .cloned()
                                .collect(),
                            None => vec![],
                        };

                        let result = serde_json::json!({
                            "total_count": filtered.len(),
                            "tools": filtered.iter().map(|t| {
                                serde_json::json!({
                                    "name": t.get("name"),
                                    "description": t.get("description"),
                                    "category": t.get("category"),
                                    "provider": t.get("provider"),
                                    "is_hot": t.get("is_hot")
                                })
                            }).collect::<Vec<_>>()
                        });
                        McpToolResult::success(
                            serde_json::to_string_pretty(&result).unwrap_or_default(),
                        )
                    }
                    Err(e) => {
                        McpToolResult::error(format!("Failed to parse discovery response: {}", e))
                    }
                }
            } else {
                McpToolResult::error(format!(
                    "Discovery service returned error: {}",
                    response.status()
                ))
            }
        }
        Err(e) => McpToolResult::error(format!("Failed to connect to discovery service: {}", e)),
    }
}

async fn handle_crossroads_get_tool_info(args: Value) -> McpToolResult {
    let tool_name = match args.get("tool_name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return McpToolResult::error("Missing required field: tool_name"),
    };

    let erebus_url = get_erebus_base_url();
    let url = format!("{}/api/discovery/tools", erebus_url);

    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let tools = data.get("tools").and_then(|t| t.as_array());
                        let tool = tools.and_then(|t| {
                            t.iter().find(|tool| {
                                tool.get("name")
                                    .and_then(|n| n.as_str())
                                    .map_or(false, |n| n == tool_name)
                            })
                        });

                        match tool {
                            Some(t) => McpToolResult::success(
                                serde_json::to_string_pretty(t).unwrap_or_default(),
                            ),
                            None => McpToolResult::error(format!(
                                "Tool '{}' not found in Crossroads",
                                tool_name
                            )),
                        }
                    }
                    Err(e) => {
                        McpToolResult::error(format!("Failed to parse discovery response: {}", e))
                    }
                }
            } else {
                McpToolResult::error(format!(
                    "Discovery service returned error: {}",
                    response.status()
                ))
            }
        }
        Err(e) => McpToolResult::error(format!("Failed to connect to discovery service: {}", e)),
    }
}

async fn handle_crossroads_list_agents(args: Value) -> McpToolResult {
    let erebus_url = get_erebus_base_url();
    let url = format!("{}/api/discovery/agents", erebus_url);

    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let agents = data.get("agents").and_then(|a| a.as_array());
                        let limited: Vec<serde_json::Value> = match agents {
                            Some(a) => a.iter().take(limit).cloned().collect(),
                            None => vec![],
                        };

                        let result = serde_json::json!({
                            "total_count": limited.len(),
                            "agents": limited.iter().map(|a| {
                                serde_json::json!({
                                    "name": a.get("name"),
                                    "description": a.get("description"),
                                    "status": a.get("status"),
                                    "capabilities": a.get("capabilities"),
                                    "tool_count": a.get("tool_count")
                                })
                            }).collect::<Vec<_>>()
                        });
                        McpToolResult::success(
                            serde_json::to_string_pretty(&result).unwrap_or_default(),
                        )
                    }
                    Err(e) => {
                        McpToolResult::error(format!("Failed to parse discovery response: {}", e))
                    }
                }
            } else {
                McpToolResult::error(format!(
                    "Discovery service returned error: {}",
                    response.status()
                ))
            }
        }
        Err(e) => McpToolResult::error(format!("Failed to connect to discovery service: {}", e)),
    }
}

async fn handle_crossroads_list_hot(_args: Value) -> McpToolResult {
    let erebus_url = get_erebus_base_url();
    let url = format!("{}/api/discovery/hot", erebus_url);

    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => McpToolResult::success(
                        serde_json::to_string_pretty(&data).unwrap_or_default(),
                    ),
                    Err(e) => {
                        McpToolResult::error(format!("Failed to parse discovery response: {}", e))
                    }
                }
            } else {
                McpToolResult::error(format!(
                    "Discovery service returned error: {}",
                    response.status()
                ))
            }
        }
        Err(e) => McpToolResult::error(format!("Failed to connect to discovery service: {}", e)),
    }
}

async fn handle_crossroads_get_stats(_args: Value) -> McpToolResult {
    let erebus_url = get_erebus_base_url();
    let url = format!("{}/api/discovery/health", erebus_url);

    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => McpToolResult::success(
                        serde_json::to_string_pretty(&data).unwrap_or_default(),
                    ),
                    Err(e) => {
                        McpToolResult::error(format!("Failed to parse discovery response: {}", e))
                    }
                }
            } else {
                McpToolResult::error(format!(
                    "Discovery service returned error: {}",
                    response.status()
                ))
            }
        }
        Err(e) => McpToolResult::error(format!("Failed to connect to discovery service: {}", e)),
    }
}

async fn execute_tool_agent(
    engrams_client: &Arc<EngramsClient>,
    name: &str,
    args: Value,
) -> McpToolResult {
    info!("Executing MCP tool (agent): {}", name);

    match name {
        "engram_create" => handle_engram_create(engrams_client, args).await,
        "engram_get" => handle_engram_get(engrams_client, args).await,
        "engram_search" => handle_engram_search(engrams_client, args).await,
        "engram_update" => handle_engram_update(engrams_client, args).await,
        "engram_delete" => handle_engram_delete(engrams_client, args).await,
        "engram_list_by_type" => handle_engram_list_by_type(engrams_client, args).await,
        "user_profile_get" => handle_user_profile_get(engrams_client, args).await,
        "user_profile_update" => handle_user_profile_update(engrams_client, args).await,
        "hecate_remember" => handle_hecate_remember(engrams_client, args).await,
        "hecate_cleanup" => handle_hecate_cleanup(engrams_client, args).await,
        "hecate_pin_engram" => handle_hecate_pin(engrams_client, args).await,
        "hecate_unpin_engram" => handle_hecate_unpin(engrams_client, args).await,
        "hecate_new_session" => handle_hecate_new_session(engrams_client, args).await,
        "hecate_list_sessions" => handle_hecate_list_sessions(engrams_client, args).await,
        "hecate_resume_session" => handle_hecate_resume_session(engrams_client, args).await,
        "hecate_delete_session" => handle_hecate_delete_session(engrams_client, args).await,
        "moros_remember" => handle_moros_remember(engrams_client, args).await,
        "moros_cleanup" => handle_hecate_cleanup(engrams_client, args).await,
        "moros_pin_engram" => handle_hecate_pin(engrams_client, args).await,
        "moros_unpin_engram" => handle_hecate_unpin(engrams_client, args).await,
        // Crossroads discovery tools (public, no wallet required)
        "crossroads_list_tools" => handle_crossroads_list_tools(args).await,
        "crossroads_get_tool_info" => handle_crossroads_get_tool_info(args).await,
        "crossroads_list_agents" => handle_crossroads_list_agents(args).await,
        "crossroads_list_hot" => handle_crossroads_list_hot(args).await,
        "crossroads_get_stats" => handle_crossroads_get_stats(args).await,
        _ => McpToolResult::error(format!("Unknown tool: {}", name)),
    }
}

// ==================== LLM Service Handlers ====================

async fn handle_llm_chat(state: &AppState, args: Value) -> McpToolResult {
    let messages = match args.get("messages").and_then(|v| v.as_array()) {
        Some(msgs) => msgs,
        None => return McpToolResult::error("Missing required field: messages"),
    };

    let messages_for_llm: Vec<serde_json::Value> = messages.clone();

    let prompt = messages
        .last()
        .and_then(|m| m.get("content"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let system_prompt = args
        .get("system_prompt")
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            messages
                .iter()
                .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("system"))
                .and_then(|m| m.get("content"))
                .and_then(|v| v.as_str())
                .map(String::from)
        });

    let model_override = args
        .get("model")
        .and_then(|v| v.as_str())
        .and_then(|m| if m == "auto" { None } else { Some(m.to_string()) });

    let max_tokens = args
        .get("max_tokens")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let temperature = args.get("temperature").and_then(|v| v.as_f64());

    let llm_request = LLMRequest {
        prompt,
        system_prompt,
        messages: Some(messages_for_llm),
        max_tokens,
        temperature,
        top_p: None,
        stop_sequences: None,
        tools: None,
        model_override,
        concise: false,
        max_chars: None,
        reasoning: None,
    };

    let agent = state.hecate_agent.read().await;
    let llm_factory = match agent.llm_factory.as_ref() {
        Some(f) => f.clone(),
        None => return McpToolResult::error("LLM service not initialized"),
    };
    drop(agent);

    let factory = llm_factory.read().await;
    match factory.generate(&llm_request, None).await {
        Ok(response) => {
            let result = serde_json::json!({
                "content": response.content,
                "model_used": response.model_used,
                "usage": response.usage,
                "finish_reason": response.finish_reason,
                "latency_ms": response.latency_ms,
                "tool_calls": response.tool_calls,
            });
            McpToolResult::success(result.to_string())
        }
        Err(e) => {
            error!("LLM chat failed: {}", e);
            McpToolResult::error(format!("LLM request failed: {}", e))
        }
    }
}

async fn handle_llm_list_models(state: &AppState, args: Value) -> McpToolResult {
    let free_only = args
        .get("free_only")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let agent = state.hecate_agent.read().await;
    let llm_factory = match agent.llm_factory.as_ref() {
        Some(f) => f.clone(),
        None => return McpToolResult::error("LLM service not initialized"),
    };
    drop(agent);

    let factory = llm_factory.read().await;

    let models = if free_only {
        factory.get_free_models().await.unwrap_or_default()
    } else {
        factory.fetch_available_models().await.unwrap_or_default()
    };

    let model_list: Vec<serde_json::Value> = models
        .iter()
        .filter_map(|m| {
            let id = m.get("id")?.as_str()?;
            let name = m.get("name").and_then(|v| v.as_str()).unwrap_or(id);
            let context = m.get("context_length").and_then(|v| v.as_i64()).unwrap_or(0);
            Some(serde_json::json!({
                "id": id,
                "name": name,
                "context_length": context,
            }))
        })
        .collect();

    let result = serde_json::json!({
        "count": model_list.len(),
        "free_only": free_only,
        "models": model_list,
    });
    McpToolResult::success(result.to_string())
}

async fn handle_llm_model_status(state: &AppState) -> McpToolResult {
    let agent = state.hecate_agent.read().await;
    let llm_factory = match agent.llm_factory.as_ref() {
        Some(f) => f.clone(),
        None => return McpToolResult::error("LLM service not initialized"),
    };
    let current_model = agent.preferred_model.clone();
    drop(agent);

    let factory = llm_factory.read().await;
    let health = factory.health_check().await.unwrap_or(serde_json::json!({"error": "health check failed"}));
    let stats = factory.get_stats().await;
    let default_free = factory.get_default_free_model();

    let result = serde_json::json!({
        "current_model": current_model,
        "default_free_model": default_free,
        "health": health,
        "stats": stats,
    });
    McpToolResult::success(result.to_string())
}

async fn handle_llm_set_model(state: &AppState, args: Value) -> McpToolResult {
    let agent_name = match args.get("agent_name").and_then(|v| v.as_str()) {
        Some(name) => name.to_string(),
        None => return McpToolResult::error("Missing required field: agent_name"),
    };
    let query = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_lowercase(),
        None => return McpToolResult::error("Missing required field: query"),
    };

    let agent = state.hecate_agent.read().await;
    let llm_factory = match agent.llm_factory.as_ref() {
        Some(f) => f.clone(),
        None => return McpToolResult::error("LLM factory not initialized"),
    };
    drop(agent);

    let factory = llm_factory.read().await;
    let models = factory.fetch_available_models().await.unwrap_or_default();

    let mut matches: Vec<(String, String, f64)> = Vec::new();
    for model in &models {
        let id = model.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let name = model.get("name").and_then(|v| v.as_str()).unwrap_or(id);
        let id_lower = id.to_lowercase();
        let name_lower = name.to_lowercase();

        if id_lower == query || name_lower == query {
            matches.push((id.to_string(), name.to_string(), 1.0));
        } else if id_lower.contains(&query) || name_lower.contains(&query) {
            let score = if id_lower.starts_with(&query) { 0.9 } else { 0.7 };
            matches.push((id.to_string(), name.to_string(), score));
        }
    }

    matches.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    if matches.is_empty() {
        return McpToolResult::error(format!(
            "No models found matching '{}'. Try 'claude', 'gpt', 'deepseek', 'llama', or 'gemini'.",
            query
        ));
    }

    let best = &matches[0];
    let previous = {
        let mut prefs = state.agent_model_preferences.write().await;
        let prev = prefs.get(&agent_name).cloned();
        prefs.insert(agent_name.clone(), best.0.clone());
        prev
    };

    let alternatives: Vec<String> = matches[1..matches.len().min(6)]
        .iter()
        .map(|m| format!("{} ({})", m.1, m.0))
        .collect();

    info!("🔄 Model preference set for {}: {} (was {:?})", agent_name, best.0, previous);

    let mut result = format!("Switched {} to model: {} ({})", agent_name, best.1, best.0);
    if let Some(prev) = previous {
        result.push_str(&format!("\nPrevious model: {}", prev));
    }
    if !alternatives.is_empty() {
        result.push_str(&format!("\nOther matches: {}", alternatives.join(", ")));
    }
    McpToolResult::success(result)
}

async fn handle_llm_get_model(state: &AppState, args: Value) -> McpToolResult {
    let agent_name = match args.get("agent_name").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => return McpToolResult::error("Missing required field: agent_name"),
    };

    let prefs = state.agent_model_preferences.read().await;
    let model = prefs.get(agent_name).cloned();

    let result = serde_json::json!({
        "agent_name": agent_name,
        "model": model.unwrap_or_else(|| "auto (no preference set)".to_string()),
    });
    McpToolResult::success(result.to_string())
}
