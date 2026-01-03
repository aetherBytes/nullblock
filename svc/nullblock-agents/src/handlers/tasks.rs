#![allow(dead_code)]

use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::Json,
};
use chrono::Utc;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::{
    database::repositories::{TaskRepository, AgentRepository},
    database::repositories::user_references::UserReferenceRepository,
    kafka::TaskLifecycleEvent,
    models::{
        TaskState,
        CreateTaskRequest, UpdateTaskRequest, TaskResponse, TaskListResponse
    },
    server::AppState,
};

#[derive(Debug, serde::Deserialize)]
pub struct TaskQuery {
    status: Option<String>,
    task_type: Option<String>,
    limit: Option<usize>,
}

// Helper function to extract user_id from wallet address
async fn get_user_id_from_wallet(
    database: &crate::database::Database,
    wallet_address: Option<&str>,
    network: Option<&str>,
) -> Option<Uuid> {
    if let (Some(wallet), Some(network)) = (wallet_address, network) {
        let user_repo = UserReferenceRepository::new(database.pool().clone());

        // Query user by source_identifier and network (the correct approach)
        match user_repo.get_by_source(wallet, network).await {
            Ok(Some(user)) => {
                // User exists, return their UUID
                info!("✅ Found existing user for wallet: {} -> {}", wallet, user.id);
                Some(user.id)
            }
            Ok(None) => {
                // User doesn't exist - this shouldn't happen if Erebus registration worked
                warn!("⚠️ No user found for wallet: {} on network: {}", wallet, network);
                warn!("⚠️ User should be registered via Erebus /api/users/register before creating tasks");
                None
            }
            Err(e) => {
                error!("❌ Failed to lookup user by source: {}", e);
                None
            }
        }
    } else {
        None
    }
}


// Wrapper function for create_task that extracts headers
pub async fn create_task_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    create_task(State(state), Json(request), headers).await
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
    headers: HeaderMap,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("📋 Creating new task: {}", request.name);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    // Create task repository
    let task_repo = TaskRepository::new(database.pool().clone());
    let agent_repo = AgentRepository::new(database.pool().clone());

    // Extract wallet address and chain from headers
    let wallet_address = headers.get("x-wallet-address")
        .and_then(|h| h.to_str().ok());
    let chain = headers.get("x-wallet-chain")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("solana"); // Default to Solana chain

    // Get user_id from wallet address
    let user_id = get_user_id_from_wallet(database, wallet_address, Some(chain)).await;

    if let Some(wallet) = wallet_address {
        info!("🔍 Creating task for wallet: {} on chain: {}", wallet, chain);
        if user_id.is_none() {
            warn!("⚠️ No user found for wallet: {}, creating task without user association", wallet);
        }
    } else {
        info!("📋 No wallet address provided, creating task without user association");
    }

    // Determine agent assignment
    let assigned_agent_id = if let Some(agent_id_str) = &request.assigned_agent_id {
        // User specified an agent - validate it exists
        match agent_repo.get_by_name_and_type(agent_id_str, "").await {
            Ok(Some(agent)) => {
                info!("✅ User assigned task to agent: {}", agent_id_str);
                Some(agent.id)
            }
            Ok(None) => {
                warn!("⚠️ Requested agent '{}' not found, defaulting to Hecate", agent_id_str);
                // Fall back to Hecate
                match agent_repo.get_by_name_and_type("hecate", "hive_mind").await {
                    Ok(Some(agent)) => Some(agent.id),
                    _ => None
                }
            }
            Err(e) => {
                warn!("⚠️ Failed to lookup agent '{}': {}, defaulting to Hecate", agent_id_str, e);
                // Fall back to Hecate
                match agent_repo.get_by_name_and_type("hecate", "hive_mind").await {
                    Ok(Some(agent)) => Some(agent.id),
                    _ => None
                }
            }
        }
    } else {
        // No agent specified - use Hecate as default orchestrator
        match agent_repo.get_by_name_and_type("hecate", "hive_mind").await {
            Ok(Some(agent)) => {
                info!("🤖 Auto-assigned task to Hecate orchestrator");
                Some(agent.id)
            }
            Ok(None) => {
                warn!("⚠️ Hecate agent not found in database, creating task without assignment");
                None
            }
            Err(e) => {
                warn!("⚠️ Failed to lookup Hecate agent: {}, creating task without assignment", e);
                None
            }
        }
    };

    // Create task in database
    match task_repo.create(&request, user_id, assigned_agent_id, wallet_address.map(|s| s.to_string())).await {
        Ok(task_entity) => {
            // Add initial message to history with task description
            let initial_message = serde_json::json!({
                "messageId": format!("msg-{}", Uuid::new_v4()),
                "role": "user",
                "parts": [{
                    "type": "text",
                    "text": request.description.clone()
                }],
                "timestamp": Utc::now().to_rfc3339(),
                "taskId": task_entity.id.to_string(),
                "contextId": task_entity.context_id.to_string(),
                "kind": "message"
            });

            if let Err(e) = task_repo.add_message_to_history(&task_entity.id.to_string(), initial_message).await {
                warn!("⚠️ Failed to add initial message to task history: {}", e);
            }

            // Convert to domain model
            match task_entity.to_domain_model() {
                Ok(task) => {
                    info!("✅ Task created successfully: {} ({})", task.name, task.id);

                    // Publish Kafka event if producer is available
                    if let Some(kafka_producer) = &state.kafka_producer {
                        let event = TaskLifecycleEvent::task_created(
                            task.id.clone(),
                            user_id,
                            task.name.clone(),
                            serde_json::to_string(&task.status).unwrap().trim_matches('"').to_string(),
                            serde_json::to_string(&task.priority).unwrap().trim_matches('"').to_string(),
                        );

                        if let Err(e) = kafka_producer.publish_task_event(event).await {
                            warn!("⚠️ Failed to publish task created event: {}", e);
                        }
                    }

                    // If auto_start is true, automatically process the task
                    if request.auto_start.unwrap_or(false) {
                        info!("🚀 Auto-starting task: {}", task.id);

                        // Create a background task to process this task
                        let state_clone = state.clone();
                        let task_id = task.id.clone();
                        tokio::spawn(async move {
                            // Small delay to ensure task is fully committed to database
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                            match process_task_internal(state_clone, task_id).await {
                                Ok(_) => info!("✅ Auto-started task processed successfully"),
                                Err(e) => error!("❌ Failed to auto-process task: {}", e),
                            }
                        });
                    }

                    Ok(Json(TaskResponse {
                        success: true,
                        data: Some(task),
                        error: None,
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to convert task entity to domain model: {}", e);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to create task".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to create task in database: {}", e);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Failed to create task".to_string()),
                timestamp: Utc::now(),
            }))
        }
    }
}

// Wrapper function for get_tasks that extracts headers
pub async fn get_tasks_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TaskQuery>,
) -> Result<Json<TaskListResponse>, StatusCode> {
    get_tasks(State(state), Query(query), headers).await
}

pub async fn get_tasks(
    State(state): State<AppState>,
    Query(query): Query<TaskQuery>,
    headers: HeaderMap,
) -> Result<Json<TaskListResponse>, StatusCode> {
    info!("📋 Fetching tasks with filters: {:?}", query);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(TaskListResponse {
                success: false,
                data: None,
                total: 0,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    // Create task repository
    let task_repo = TaskRepository::new(database.pool().clone());

    // Extract wallet address and chain from headers
    let wallet_address = headers.get("x-wallet-address")
        .and_then(|h| h.to_str().ok());
    let chain = headers.get("x-wallet-chain")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("solana"); // Default to Solana chain

    // Get user_id from wallet address
    let user_id = get_user_id_from_wallet(database, wallet_address, Some(chain)).await;
    
    if let Some(wallet) = wallet_address {
        info!("🔍 Looking up tasks for wallet: {} on chain: {}", wallet, chain);
        if user_id.is_none() {
            warn!("⚠️ No user found for wallet: {}, returning empty task list", wallet);
        }
    } else {
        info!("📋 No wallet address provided, returning all tasks (admin mode)");
    }

    // Fetch tasks from database
    match task_repo.list(
        user_id,
        query.status.as_deref(),
        query.task_type.as_deref(),
        query.limit.map(|l| l as i64),
    ).await {
        Ok(task_entities) => {
            // Convert to domain models
            let mut tasks = Vec::new();
            for entity in task_entities {
                match entity.to_domain_model() {
                    Ok(task) => tasks.push(task),
                    Err(e) => {
                        warn!("⚠️ Failed to convert task entity to domain model: {}", e);
                    }
                }
            }

            let total = tasks.len();
            info!("✅ Returning {} tasks", total);

            Ok(Json(TaskListResponse {
                success: true,
                data: Some(tasks),
                total,
                error: None,
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to fetch tasks from database: {}", e);
            Ok(Json(TaskListResponse {
                success: false,
                data: None,
                total: 0,
                error: Some("Failed to fetch tasks".to_string()),
                timestamp: Utc::now(),
            }))
        }
    }
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("📋 Fetching task: {}", task_id);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    // Create task repository
    let task_repo = TaskRepository::new(database.pool().clone());

    // Fetch task from database
    match task_repo.get_by_id(&task_id).await {
        Ok(Some(task_entity)) => {
            match task_entity.to_domain_model() {
                Ok(task) => {
                    info!("✅ Found task: {}", task.name);
                    Ok(Json(TaskResponse {
                        success: true,
                        data: Some(task),
                        error: None,
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to convert task entity to domain model: {}", e);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to fetch task".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Ok(None) => {
            warn!("⚠️ Task not found: {}", task_id);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some(format!("Task not found: {}", task_id)),
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to fetch task from database: {}", e);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Failed to fetch task".to_string()),
                timestamp: Utc::now(),
            }))
        }
    }
}

pub async fn update_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(update_request): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("📝 Updating task: {}", task_id);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    let task_repo = TaskRepository::new(database.pool().clone());

    // Update the task using the repository
    match task_repo.update(&task_id, &update_request).await {
        Ok(Some(task_entity)) => {
            // Convert entity to domain model
            match task_entity.to_domain_model() {
                Ok(task) => {
                    info!("✅ Task updated successfully: {}", task.name);

                    Ok(Json(TaskResponse {
                        success: true,
                        data: Some(task),
                        error: None,
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to convert task entity: {}", e);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to convert task data".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Ok(None) => {
            warn!("⚠️ Task not found for update: {}", task_id);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some(format!("Task not found: {}", task_id)),
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to update task: {}", e);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database operation failed".to_string()),
                timestamp: Utc::now(),
            }))
        }
    }
}

pub async fn delete_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("🗑️ Deleting task: {}", task_id);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    let task_repo = TaskRepository::new(database.pool().clone());

    // Delete the task using the repository
    match task_repo.delete(&task_id).await {
        Ok(Some(task_entity)) => {
            // Convert entity to domain model
            match task_entity.to_domain_model() {
                Ok(task) => {
                    info!("✅ Task deleted successfully: {}", task.name);

                    Ok(Json(TaskResponse {
                        success: true,
                        data: Some(task),
                        error: None,
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to convert task entity: {}", e);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to convert task data".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Ok(None) => {
            warn!("⚠️ Task not found for deletion: {}", task_id);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some(format!("Task not found: {}", task_id)),
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to delete task: {}", e);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database operation failed".to_string()),
                timestamp: Utc::now(),
            }))
        }
    }
}

pub async fn start_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("▶️ Starting task: {}", task_id);

    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    let task_repo = TaskRepository::new(database.pool().clone());

    match task_repo.update_status(&task_id, TaskState::Working).await {
        Ok(Some(task_entity)) => {
            match task_entity.to_domain_model() {
                Ok(task) => {
                    info!("✅ Task started successfully: {}", task.name);

                    // Publish Kafka event
                    if let Some(kafka_producer) = &state.kafka_producer {
                        let event = TaskLifecycleEvent::task_status_changed(
                            task.id.clone(),
                            None, // user_id
                            None, // agent_id
                            task.name.clone(),
                            "submitted".to_string(), // previous A2A state
                            "working".to_string(),   // new A2A state
                            serde_json::to_string(&task.priority).unwrap().trim_matches('"').to_string(),
                            task.progress,
                        );

                        if let Err(e) = kafka_producer.publish_task_event(event).await {
                            warn!("⚠️ Failed to publish task started event: {}", e);
                        }
                    }

                    Ok(Json(TaskResponse {
                        success: true,
                        data: Some(task),
                        error: None,
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to convert task entity: {}", e);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to start task".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Ok(None) => {
            warn!("⚠️ Task not found for start: {}", task_id);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some(format!("Task not found: {}", task_id)),
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to start task: {}", e);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Failed to start task".to_string()),
                timestamp: Utc::now(),
            }))
        }
    }
}

pub async fn pause_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("⏸️ Pausing task: {}", task_id);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    let task_repo = TaskRepository::new(database.pool().clone());

    match task_repo.update_status(&task_id, TaskState::InputRequired).await {
        Ok(Some(task_entity)) => {
            match task_entity.to_domain_model() {
                Ok(task) => {
                    info!("✅ Task paused successfully: {}", task.name);

                    Ok(Json(TaskResponse {
                        success: true,
                        data: Some(task),
                        error: None,
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to convert task entity: {}", e);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to convert task data".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Ok(None) => {
            warn!("⚠️ Task not found for pause: {}", task_id);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some(format!("Task not found: {}", task_id)),
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to pause task: {}", e);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database operation failed".to_string()),
                timestamp: Utc::now(),
            }))
        }
    }
}

pub async fn resume_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("▶️ Resuming task: {}", task_id);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    let task_repo = TaskRepository::new(database.pool().clone());

    match task_repo.update_status(&task_id, TaskState::Working).await {
        Ok(Some(task_entity)) => {
            match task_entity.to_domain_model() {
                Ok(task) => {
                    info!("✅ Task resumed successfully: {}", task.name);

                    Ok(Json(TaskResponse {
                        success: true,
                        data: Some(task),
                        error: None,
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to convert task entity: {}", e);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to convert task data".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Ok(None) => {
            warn!("⚠️ Task not found for resume: {}", task_id);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some(format!("Task not found: {}", task_id)),
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to resume task: {}", e);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database operation failed".to_string()),
                timestamp: Utc::now(),
            }))
        }
    }
}

pub async fn cancel_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("🚫 Cancelling task: {}", task_id);

    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    let task_repo = TaskRepository::new(database.pool().clone());

    match task_repo.update_status(&task_id, TaskState::Canceled).await {
        Ok(Some(task_entity)) => {
            match task_entity.to_domain_model() {
                Ok(task) => {
                    info!("✅ Task cancelled successfully: {}", task.name);
                    Ok(Json(TaskResponse {
                        success: true,
                        data: Some(task),
                        error: None,
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to convert task entity: {}", e);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to convert task data".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Ok(None) => {
            warn!("⚠️ Task not found for cancellation: {}", task_id);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some(format!("Task not found: {}", task_id)),
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to cancel task: {}", e);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database operation failed".to_string()),
                timestamp: Utc::now(),
            }))
        }
    }
}

pub async fn retry_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("🔄 Retrying task: {}", task_id);

    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database connection not available".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    let task_repo = TaskRepository::new(database.pool().clone());

    match task_repo.update_status(&task_id, TaskState::Working).await {
        Ok(Some(task_entity)) => {
            match task_entity.to_domain_model() {
                Ok(task) => {
                    info!("✅ Task retried successfully: {}", task.name);
                    Ok(Json(TaskResponse {
                        success: true,
                        data: Some(task),
                        error: None,
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to convert task entity: {}", e);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to convert task data".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Ok(None) => {
            warn!("⚠️ Task not found for retry: {}", task_id);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some(format!("Task not found: {}", task_id)),
                timestamp: Utc::now(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to retry task: {}", e);
            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Database operation failed".to_string()),
                timestamp: Utc::now(),
            }))
        }
    }
}

// Internal function for processing tasks (used by both public endpoint and auto-start)
async fn process_task_internal(
    state: AppState,
    task_id: String,
) -> Result<TaskResponse, String> {
    info!("🎯 Processing task internally: {}", task_id);

    // Check if we have database connection
    let database = match &state.database {
        Some(db) => db,
        None => {
            error!("❌ Database connection not available");
            return Err("Database connection not available".to_string());
        }
    };

    let task_repo = TaskRepository::new(database.pool().clone());
    let agent_repo = AgentRepository::new(database.pool().clone());

    // Get the task to process
    let task_entity = match task_repo.get_by_id(&task_id).await {
        Ok(Some(task)) => task,
        Ok(None) => {
            warn!("⚠️ Task not found: {}", task_id);
            return Err(format!("Task not found: {}", task_id));
        }
        Err(e) => {
            error!("❌ Failed to fetch task: {}", e);
            return Err("Failed to fetch task".to_string());
        }
    };

    // Check if task is in a processable state - if it's submitted, start it automatically
    // Valid A2A states: submitted, working, input-required, completed, canceled, failed, rejected, auth-required, unknown
    if task_entity.status == "submitted" {
        info!("🚀 Auto-starting task {} before processing (status: submitted → working)", task_id);
        match task_repo.update_status(&task_id, crate::models::TaskState::Working).await {
            Ok(Some(updated_task)) => {
                info!("✅ Task {} automatically started", task_id);

                // Publish Kafka event for auto-start
                if let Some(kafka_producer) = &state.kafka_producer {
                    if let Ok(task_model) = updated_task.to_domain_model() {
                        let event = TaskLifecycleEvent::task_status_changed(
                            task_id.clone(),
                            None, // No user_id available in Task domain model
                            None, // No agent_id yet
                            task_model.name.clone(),
                            "submitted".to_string(), // previous A2A state
                            "working".to_string(),   // new A2A state
                            format!("{:?}", task_model.priority), // Use Debug format
                            0, // progress: u8 (0% when starting)
                        );
                        let _ = kafka_producer.publish_task_event(event).await;
                    }
                }
            }
            Ok(None) => {
                warn!("⚠️ Task {} not found when auto-starting", task_id);
                return Err(format!("Task not found when starting: {}", task_id));
            }
            Err(e) => {
                error!("❌ Failed to auto-start task {}: {}", task_id, e);
                return Err(format!("Failed to start task: {}", e));
            }
        }
    } else if task_entity.status != "working" {
        // Task must be in 'submitted' (to auto-start) or 'working' (already started) to process
        warn!("⚠️ Task {} is not in a processable state: {}", task_id, task_entity.status);
        return Err(format!("Task must be in 'submitted' or 'working' state to process. Current state: {}", task_entity.status));
    }

    // Execute the task using Hecate
    let mut hecate_agent = state.hecate_agent.write().await;
    let task_description = task_entity.description.as_deref().unwrap_or(&task_entity.name);

    match hecate_agent.execute_task(&task_id, task_description, &task_repo, &agent_repo).await {
        Ok(_result) => {
            info!("✅ Task {} processed successfully", task_id);

            // Get updated task from database
            match task_repo.get_by_id(&task_id).await {
                Ok(Some(updated_task)) => {
                    match updated_task.to_domain_model() {
                        Ok(task_model) => {
                            // Publish Kafka event for task completion
                            if let Some(kafka_producer) = &state.kafka_producer {
                                let event = TaskLifecycleEvent::task_status_changed(
                                    task_id.clone(),
                                    None, // user_id
                                    hecate_agent.get_agent_id(),
                                    task_model.name.clone(),
                                    "working".to_string(),   // previous A2A state
                                    "completed".to_string(), // final A2A state
                                    serde_json::to_string(&task_model.priority).unwrap().trim_matches('"').to_string(),
                                    task_model.progress,
                                );

                                if let Err(e) = kafka_producer.publish_task_event(event).await {
                                    warn!("⚠️ Failed to publish task completed event: {}", e);
                                }
                            }

                            Ok(TaskResponse {
                                success: true,
                                data: Some(task_model),
                                error: None,
                                timestamp: Utc::now(),
                            })
                        }
                        Err(e) => {
                            error!("❌ Failed to convert processed task entity: {}", e);
                            Err("Failed to retrieve processed task".to_string())
                        }
                    }
                }
                Ok(None) => {
                    error!("❌ Task disappeared after processing: {}", task_id);
                    Err("Failed to retrieve processed task".to_string())
                }
                Err(e) => {
                    error!("❌ Failed to fetch processed task: {}", e);
                    Err("Failed to retrieve processed task".to_string())
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to process task {}: {}", task_id, e);
            Err(format!("Failed to process task: {}", e))
        }
    }
}

pub async fn process_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    match process_task_internal(state, task_id).await {
        Ok(response) => Ok(Json(response)),
        Err(error) => Ok(Json(TaskResponse {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }))
    }
}

// Utility function to convert wallet address to deterministic UUID
fn wallet_to_uuid(wallet_address: &str, chain: &str) -> Uuid {
    use sha2::{Sha256, Digest};

    // Create input string combining wallet and chain for uniqueness
    let input = format!("{}:{}", wallet_address.to_lowercase(), chain.to_lowercase());

    // Generate SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();

    // Convert first 16 bytes of hash to UUID
    // This ensures deterministic UUIDs for the same wallet+chain combination
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes.copy_from_slice(&hash[0..16]);

    // Set version (4) and variant bits to create a valid UUID v4
    uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x40; // Version 4
    uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80; // Variant 10

    Uuid::from_bytes(uuid_bytes)
}

// Migration function to update existing users to wallet-derived UUIDs
async fn migrate_existing_users_to_wallet_uuids(
    database: &crate::database::Database,
) -> Result<(), String> {
    info!("🔄 Starting migration of existing users to wallet-derived UUIDs");

    let user_repo = UserReferenceRepository::new(database.pool().clone());

    // Get all existing users
    match user_repo.list_active(None).await {
        Ok(existing_users) => {
            let mut migrated_count = 0;
            let mut failed_count = 0;

            for user_entity in existing_users {
                if let (Some(source_identifier), Some(network)) = (&user_entity.source_identifier, &user_entity.network) {
                    // Calculate what the UUID should be
                    let correct_uuid = wallet_to_uuid(source_identifier, network);

                    if user_entity.id != correct_uuid {
                        info!("🔄 Migrating user {} -> {}", user_entity.id, correct_uuid);

                        // Create new user with correct UUID
                        let new_user_ref = crate::models::UserReference {
                            id: correct_uuid,
                            source_identifier: source_identifier.to_string(),
                            network: network.to_string(),
                            source_type: serde_json::json!({
                                "type": "web3_wallet",
                                "provider": "web3",
                                "metadata": {}
                            }),
                            wallet_type: Some("web3".to_string()),
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        };

                        match user_repo.create(&new_user_ref).await {
                            Ok(_) => {
                                info!("✅ Created new user with correct UUID: {}", correct_uuid);
                                migrated_count += 1;

                                // TODO: Update any existing tasks to reference the new user_id
                                // This would require a task repository update as well
                            }
                            Err(e) => {
                                error!("❌ Failed to create new user {}: {}", correct_uuid, e);
                                failed_count += 1;
                            }
                        }
                    } else {
                        info!("✅ User {} already has correct UUID", user_entity.id);
                    }
                } else {
                    warn!("⚠️ User {} missing wallet_address or chain", user_entity.id);
                    failed_count += 1;
                }
            }

            info!("🎉 Migration completed: {} migrated, {} failed", migrated_count, failed_count);
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to list existing users: {}", e);
            Err(format!("Failed to list users: {}", e))
        }
    }
}