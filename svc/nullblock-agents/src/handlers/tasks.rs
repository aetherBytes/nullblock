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
        TaskStatus,
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
    chain: Option<&str>,
) -> Option<Uuid> {
    if let (Some(wallet), Some(chain)) = (wallet_address, chain) {
        let user_repo = UserReferenceRepository::new(database.pool().clone());
        match user_repo.get_by_wallet(wallet, chain).await {
            Ok(Some(user_ref)) => Some(user_ref.id),
            Ok(None) => {
                warn!("⚠️ User not found for wallet: {} on chain: {}", wallet, chain);
                None
            }
            Err(e) => {
                error!("❌ Failed to lookup user by wallet: {}", e);
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

    // Get Hecate agent ID for task assignment
    let hecate_agent_id = match agent_repo.get_by_name_and_type("hecate", "conversational").await {
        Ok(Some(agent)) => Some(agent.id),
        Ok(None) => {
            warn!("⚠️ Hecate agent not found in database, creating task without assignment");
            None
        }
        Err(e) => {
            warn!("⚠️ Failed to lookup Hecate agent: {}, creating task without assignment", e);
            None
        }
    };

    // Create task in database
    match task_repo.create(&request, user_id, hecate_agent_id).await {
        Ok(task_entity) => {
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

    match task_repo.update_status(&task_id, TaskStatus::Running).await {
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
                            "created".to_string(), // assume previous status
                            "running".to_string(),
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

    match task_repo.update_status(&task_id, TaskStatus::Paused).await {
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

    match task_repo.update_status(&task_id, TaskStatus::Running).await {
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

    match task_repo.update_status(&task_id, TaskStatus::Cancelled).await {
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

    match task_repo.update_status(&task_id, TaskStatus::Running).await {
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

pub async fn process_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("🎯 Processing task: {}", task_id);

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
    let agent_repo = AgentRepository::new(database.pool().clone());

    // Get the task to process
    let task_entity = match task_repo.get_by_id(&task_id).await {
        Ok(Some(task)) => task,
        Ok(None) => {
            warn!("⚠️ Task not found: {}", task_id);
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some(format!("Task not found: {}", task_id)),
                timestamp: Utc::now(),
            }));
        }
        Err(e) => {
            error!("❌ Failed to fetch task: {}", e);
            return Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some("Failed to fetch task".to_string()),
                timestamp: Utc::now(),
            }));
        }
    };

    // Check if task is in a processable state
    if task_entity.status != "running" {
        warn!("⚠️ Task {} is not in running state: {}", task_id, task_entity.status);
        return Ok(Json(TaskResponse {
            success: false,
            data: None,
            error: Some(format!("Task must be in 'running' state to process. Current state: {}", task_entity.status)),
            timestamp: Utc::now(),
        }));
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
                            // Publish Kafka event
                            if let Some(kafka_producer) = &state.kafka_producer {
                                let event = TaskLifecycleEvent::task_status_changed(
                                    task_id.clone(),
                                    None, // user_id
                                    hecate_agent.get_agent_id(),
                                    task_model.name.clone(),
                                    "running".to_string(),
                                    "processed".to_string(), // Custom status for processed tasks
                                    serde_json::to_string(&task_model.priority).unwrap().trim_matches('"').to_string(),
                                    task_model.progress,
                                );

                                if let Err(e) = kafka_producer.publish_task_event(event).await {
                                    warn!("⚠️ Failed to publish task processed event: {}", e);
                                }
                            }

                            Ok(Json(TaskResponse {
                                success: true,
                                data: Some(task_model),
                                error: None,
                                timestamp: Utc::now(),
                            }))
                        }
                        Err(e) => {
                            error!("❌ Failed to convert processed task entity: {}", e);
                            Ok(Json(TaskResponse {
                                success: false,
                                data: None,
                                error: Some("Failed to retrieve processed task".to_string()),
                                timestamp: Utc::now(),
                            }))
                        }
                    }
                }
                Ok(None) => {
                    error!("❌ Task disappeared after processing: {}", task_id);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Task not found after processing".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
                Err(e) => {
                    error!("❌ Failed to fetch processed task: {}", e);
                    Ok(Json(TaskResponse {
                        success: false,
                        data: None,
                        error: Some("Failed to retrieve processed task".to_string()),
                        timestamp: Utc::now(),
                    }))
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to process task {}: {}", task_id, e);

            // Check if it was already actioned
            if e.to_string().contains("already actioned") {
                return Ok(Json(TaskResponse {
                    success: false,
                    data: None,
                    error: Some("Task has already been processed".to_string()),
                    timestamp: Utc::now(),
                }));
            }

            Ok(Json(TaskResponse {
                success: false,
                data: None,
                error: Some(format!("Task processing failed: {}", e)),
                timestamp: Utc::now(),
            }))
        }
    }
}