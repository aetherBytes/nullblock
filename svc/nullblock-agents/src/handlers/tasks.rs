use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use tracing::{info, warn, error};

use crate::{
    database::repositories::TaskRepository,
    kafka::TaskLifecycleEvent,
    models::{
        Task, TaskStatus,
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


pub async fn create_task(
    State(state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
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

    // For now, we don't have user context from the request
    // TODO: Extract user_id from JWT token or session when authentication is implemented
    let user_id = None;

    // Create task in database
    match task_repo.create(&request, user_id).await {
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
                            format!("{:?}", task.status).to_lowercase(),
                            format!("{:?}", task.priority).to_lowercase(),
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

pub async fn get_tasks(
    State(state): State<AppState>,
    Query(query): Query<TaskQuery>,
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

    // For now, we don't filter by user_id
    // TODO: Extract user_id from JWT token or session when authentication is implemented
    let user_id = None;

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
                            format!("{:?}", task.priority).to_lowercase(),
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