use serde_json::Value;
use tracing::info;

use super::tools::McpToolResult;
use crate::engrams::{CreateEngramRequest, EngramsClient, SearchRequest};
use std::sync::Arc;

pub async fn execute_tool(
    engrams_client: &Arc<EngramsClient>,
    name: &str,
    args: Value,
) -> McpToolResult {
    info!("Executing MCP tool: {}", name);

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
