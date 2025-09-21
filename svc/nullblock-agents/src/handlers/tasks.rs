use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::{
    models::{
        Task, TaskStatus, TaskPriority, TaskType, TaskCategory, TaskOutcome,
        CreateTaskRequest, UpdateTaskRequest, TaskResponse, TaskListResponse
    },
    server::AppState,
};

// In-memory task storage for session-based persistence
static mut TASKS: Option<Vec<Task>> = None;
static mut TASK_COUNTER: u64 = 0;

fn get_tasks_storage() -> &'static mut Vec<Task> {
    unsafe {
        if TASKS.is_none() {
            TASKS = Some(Vec::new());
        }
        TASKS.as_mut().unwrap()
    }
}

fn generate_task_id() -> String {
    unsafe {
        TASK_COUNTER += 1;
        format!("task_{}", TASK_COUNTER)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct TaskQuery {
    status: Option<String>,
    task_type: Option<String>,
    limit: Option<usize>,
}

pub async fn create_task(
    State(_state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("📋 Creating new task: {}", request.name);

    let now = Utc::now();
    let task_id = generate_task_id();

    let status = if request.auto_start.unwrap_or(false) {
        TaskStatus::Running
    } else {
        TaskStatus::Created
    };

    let task = Task {
        id: task_id.clone(),
        name: request.name,
        description: request.description,
        task_type: request.task_type,
        category: request.category.unwrap_or(TaskCategory::UserAssigned),
        status: status.clone(),
        priority: request.priority.unwrap_or(TaskPriority::Medium),
        created_at: now,
        updated_at: now,
        started_at: if status == TaskStatus::Running { Some(now) } else { None },
        completed_at: None,
        progress: 0,
        estimated_duration: None,
        actual_duration: None,
        sub_tasks: Vec::new(),
        dependencies: request.dependencies.unwrap_or_default(),
        context: HashMap::new(),
        parameters: request.parameters.unwrap_or_default(),
        outcome: None,
        logs: Vec::new(),
        triggers: Vec::new(),
        assigned_agent: None,
        auto_retry: true,
        max_retries: 3,
        current_retries: 0,
        required_capabilities: Vec::new(),
        user_approval_required: request.user_approval_required.unwrap_or(false),
        user_notifications: true,
    };

    // Store task in session storage
    let tasks = get_tasks_storage();
    tasks.push(task.clone());

    info!("✅ Task created successfully: {} ({})", task.name, task.id);

    Ok(Json(TaskResponse {
        success: true,
        data: Some(task),
        error: None,
        timestamp: Utc::now(),
    }))
}

pub async fn get_tasks(
    State(_state): State<AppState>,
    Query(query): Query<TaskQuery>,
) -> Result<Json<TaskListResponse>, StatusCode> {
    info!("📋 Fetching tasks with filters: {:?}", query);

    let tasks = get_tasks_storage();
    let mut filtered_tasks = tasks.clone();

    // Apply filters
    if let Some(status_filter) = query.status {
        filtered_tasks.retain(|task| {
            format!("{:?}", task.status).to_lowercase() == status_filter.to_lowercase()
        });
    }

    if let Some(type_filter) = query.task_type {
        filtered_tasks.retain(|task| {
            format!("{:?}", task.task_type).to_lowercase() == type_filter.to_lowercase()
        });
    }

    // Apply limit
    if let Some(limit) = query.limit {
        filtered_tasks.truncate(limit);
    }

    let total = filtered_tasks.len();

    info!("✅ Returning {} tasks (total: {})", filtered_tasks.len(), tasks.len());

    Ok(Json(TaskListResponse {
        success: true,
        data: Some(filtered_tasks),
        total,
        error: None,
        timestamp: Utc::now(),
    }))
}

pub async fn get_task(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("📋 Fetching task: {}", task_id);

    let tasks = get_tasks_storage();

    if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
        info!("✅ Found task: {}", task.name);
        Ok(Json(TaskResponse {
            success: true,
            data: Some(task.clone()),
            error: None,
            timestamp: Utc::now(),
        }))
    } else {
        warn!("⚠️ Task not found: {}", task_id);
        Ok(Json(TaskResponse {
            success: false,
            data: None,
            error: Some(format!("Task not found: {}", task_id)),
            timestamp: Utc::now(),
        }))
    }
}

pub async fn update_task(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
    Json(update_request): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("📝 Updating task: {}", task_id);

    let tasks = get_tasks_storage();

    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
        // Apply updates
        if let Some(name) = update_request.name {
            task.name = name;
        }
        if let Some(description) = update_request.description {
            task.description = description;
        }
        if let Some(status) = update_request.status {
            task.status = status;
        }
        if let Some(priority) = update_request.priority {
            task.priority = priority;
        }
        if let Some(progress) = update_request.progress {
            task.progress = progress;
        }
        if let Some(parameters) = update_request.parameters {
            task.parameters = parameters;
        }
        if let Some(started_at) = update_request.started_at {
            task.started_at = Some(started_at);
        }
        if let Some(completed_at) = update_request.completed_at {
            task.completed_at = Some(completed_at);
        }
        if let Some(outcome) = update_request.outcome {
            task.outcome = Some(outcome);
        }

        task.updated_at = Utc::now();

        info!("✅ Task updated successfully: {}", task.name);

        Ok(Json(TaskResponse {
            success: true,
            data: Some(task.clone()),
            error: None,
            timestamp: Utc::now(),
        }))
    } else {
        warn!("⚠️ Task not found for update: {}", task_id);
        Ok(Json(TaskResponse {
            success: false,
            data: None,
            error: Some(format!("Task not found: {}", task_id)),
            timestamp: Utc::now(),
        }))
    }
}

pub async fn delete_task(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("🗑️ Deleting task: {}", task_id);

    let tasks = get_tasks_storage();

    if let Some(pos) = tasks.iter().position(|t| t.id == task_id) {
        let removed_task = tasks.remove(pos);
        info!("✅ Task deleted successfully: {}", removed_task.name);

        Ok(Json(TaskResponse {
            success: true,
            data: Some(removed_task),
            error: None,
            timestamp: Utc::now(),
        }))
    } else {
        warn!("⚠️ Task not found for deletion: {}", task_id);
        Ok(Json(TaskResponse {
            success: false,
            data: None,
            error: Some(format!("Task not found: {}", task_id)),
            timestamp: Utc::now(),
        }))
    }
}

pub async fn start_task(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("▶️ Starting task: {}", task_id);

    let tasks = get_tasks_storage();

    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
        task.status = TaskStatus::Running;
        task.started_at = Some(Utc::now());
        task.updated_at = Utc::now();
        task.progress = 0;

        info!("✅ Task started successfully: {}", task.name);

        Ok(Json(TaskResponse {
            success: true,
            data: Some(task.clone()),
            error: None,
            timestamp: Utc::now(),
        }))
    } else {
        warn!("⚠️ Task not found for start: {}", task_id);
        Ok(Json(TaskResponse {
            success: false,
            data: None,
            error: Some(format!("Task not found: {}", task_id)),
            timestamp: Utc::now(),
        }))
    }
}

pub async fn pause_task(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("⏸️ Pausing task: {}", task_id);

    let tasks = get_tasks_storage();

    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
        task.status = TaskStatus::Paused;
        task.updated_at = Utc::now();

        info!("✅ Task paused successfully: {}", task.name);

        Ok(Json(TaskResponse {
            success: true,
            data: Some(task.clone()),
            error: None,
            timestamp: Utc::now(),
        }))
    } else {
        warn!("⚠️ Task not found for pause: {}", task_id);
        Ok(Json(TaskResponse {
            success: false,
            data: None,
            error: Some(format!("Task not found: {}", task_id)),
            timestamp: Utc::now(),
        }))
    }
}

pub async fn resume_task(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("▶️ Resuming task: {}", task_id);

    let tasks = get_tasks_storage();

    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
        task.status = TaskStatus::Running;
        task.updated_at = Utc::now();

        info!("✅ Task resumed successfully: {}", task.name);

        Ok(Json(TaskResponse {
            success: true,
            data: Some(task.clone()),
            error: None,
            timestamp: Utc::now(),
        }))
    } else {
        warn!("⚠️ Task not found for resume: {}", task_id);
        Ok(Json(TaskResponse {
            success: false,
            data: None,
            error: Some(format!("Task not found: {}", task_id)),
            timestamp: Utc::now(),
        }))
    }
}

pub async fn cancel_task(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("🚫 Cancelling task: {}", task_id);

    let tasks = get_tasks_storage();

    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
        task.status = TaskStatus::Cancelled;
        task.updated_at = Utc::now();

        info!("✅ Task cancelled successfully: {}", task.name);

        Ok(Json(TaskResponse {
            success: true,
            data: Some(task.clone()),
            error: None,
            timestamp: Utc::now(),
        }))
    } else {
        warn!("⚠️ Task not found for cancel: {}", task_id);
        Ok(Json(TaskResponse {
            success: false,
            data: None,
            error: Some(format!("Task not found: {}", task_id)),
            timestamp: Utc::now(),
        }))
    }
}

pub async fn retry_task(
    State(_state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    info!("🔄 Retrying task: {}", task_id);

    let tasks = get_tasks_storage();

    if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
        task.status = TaskStatus::Running;
        task.current_retries = 0;
        task.progress = 0;
        task.started_at = Some(Utc::now());
        task.updated_at = Utc::now();
        task.outcome = None;

        info!("✅ Task retry initiated successfully: {}", task.name);

        Ok(Json(TaskResponse {
            success: true,
            data: Some(task.clone()),
            error: None,
            timestamp: Utc::now(),
        }))
    } else {
        warn!("⚠️ Task not found for retry: {}", task_id);
        Ok(Json(TaskResponse {
            success: false,
            data: None,
            error: Some(format!("Task not found: {}", task_id)),
            timestamp: Utc::now(),
        }))
    }
}