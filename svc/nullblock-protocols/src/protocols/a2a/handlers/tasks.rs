use axum::{
    extract::{Path, State},
    Json,
};
use tracing::{error, info};

use crate::errors::ProtocolError;
use crate::protocols::a2a::types::{
    Task, TaskCancelRequest, TaskCancelResponse, TaskListRequest, TaskListResponse,
};
use crate::server::AppState;

pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Task>, ProtocolError> {
    info!("📋 A2A: Fetching task {}", task_id);

    let url = format!("{}/tasks/{}", state.agents_service_url, task_id);

    match state.http_client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        if let Some(task_data) = json_response.get("data") {
                            match serde_json::from_value::<Task>(task_data.clone()) {
                                Ok(task) => {
                                    info!("✅ A2A: Task {} retrieved successfully", task_id);
                                    Ok(Json(task))
                                }
                                Err(e) => {
                                    error!("❌ A2A: Failed to parse task response: {}", e);
                                    Err(ProtocolError::InternalError(format!(
                                        "Failed to parse task: {}",
                                        e
                                    )))
                                }
                            }
                        } else {
                            error!("❌ A2A: No data field in response");
                            Err(ProtocolError::TaskNotFound(format!(
                                "Task {} not found",
                                task_id
                            )))
                        }
                    }
                    Err(e) => {
                        error!("❌ A2A: Failed to parse JSON response: {}", e);
                        Err(ProtocolError::InternalError(format!(
                            "Invalid response format: {}",
                            e
                        )))
                    }
                }
            } else if response.status() == 404 {
                Err(ProtocolError::TaskNotFound(format!(
                    "Task {} not found",
                    task_id
                )))
            } else {
                error!(
                    "❌ A2A: Task fetch failed with status {}",
                    response.status()
                );
                Err(ProtocolError::InternalError(format!(
                    "Failed to fetch task: {}",
                    response.status()
                )))
            }
        }
        Err(e) => {
            error!("❌ A2A: HTTP request failed: {}", e);
            Err(ProtocolError::InternalError(format!(
                "Request failed: {}",
                e
            )))
        }
    }
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Json(request): Json<TaskListRequest>,
) -> Result<Json<TaskListResponse>, ProtocolError> {
    info!("📋 A2A: Listing tasks with filters");

    let mut url = format!("{}/tasks", state.agents_service_url);
    let mut query_params = vec![];

    if let Some(status) = &request.status {
        query_params.push(format!("status={}", status));
    }
    if let Some(limit) = request.limit {
        query_params.push(format!("limit={}", limit));
    }

    if !query_params.is_empty() {
        url.push('?');
        url.push_str(&query_params.join("&"));
    }

    match state.http_client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        if let Some(tasks_data) = json_response.get("data") {
                            match serde_json::from_value::<Vec<Task>>(tasks_data.clone()) {
                                Ok(tasks) => {
                                    info!("✅ A2A: Listed {} tasks", tasks.len());
                                    let total = tasks.len() as u32;
                                    let next_offset =
                                        request.offset.and_then(|o| request.limit.map(|l| o + l));
                                    Ok(Json(TaskListResponse {
                                        tasks,
                                        total,
                                        next_offset,
                                    }))
                                }
                                Err(e) => {
                                    error!("❌ A2A: Failed to parse tasks response: {}", e);
                                    Err(ProtocolError::InternalError(format!(
                                        "Failed to parse tasks: {}",
                                        e
                                    )))
                                }
                            }
                        } else {
                            Ok(Json(TaskListResponse {
                                tasks: vec![],
                                total: 0,
                                next_offset: None,
                            }))
                        }
                    }
                    Err(e) => {
                        error!("❌ A2A: Failed to parse JSON response: {}", e);
                        Err(ProtocolError::InternalError(format!(
                            "Invalid response format: {}",
                            e
                        )))
                    }
                }
            } else {
                error!("❌ A2A: Task list failed with status {}", response.status());
                Err(ProtocolError::InternalError(format!(
                    "Failed to list tasks: {}",
                    response.status()
                )))
            }
        }
        Err(e) => {
            error!("❌ A2A: HTTP request failed: {}", e);
            Err(ProtocolError::InternalError(format!(
                "Request failed: {}",
                e
            )))
        }
    }
}

pub async fn cancel_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    Json(_request): Json<TaskCancelRequest>,
) -> Result<Json<TaskCancelResponse>, ProtocolError> {
    info!("🚫 A2A: Cancelling task {}", task_id);

    let url = format!(
        "{}/api/agents/tasks/{}/cancel",
        state.agents_service_url, task_id
    );

    match state.http_client.post(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        if let Some(_task_data) = json_response.get("data") {
                            info!("✅ A2A: Task {} cancelled successfully", task_id);
                            Ok(Json(TaskCancelResponse {
                                task_id,
                                cancelled: true,
                                message: Some("Task cancelled successfully".to_string()),
                            }))
                        } else {
                            error!("❌ A2A: No data field in response");
                            Err(ProtocolError::TaskNotFound(format!(
                                "Task {} not found",
                                task_id
                            )))
                        }
                    }
                    Err(e) => {
                        error!("❌ A2A: Failed to parse JSON response: {}", e);
                        Err(ProtocolError::InternalError(format!(
                            "Invalid response format: {}",
                            e
                        )))
                    }
                }
            } else if response.status() == 404 {
                Err(ProtocolError::TaskNotFound(format!(
                    "Task {} not found",
                    task_id
                )))
            } else {
                error!(
                    "❌ A2A: Task cancel failed with status {}",
                    response.status()
                );
                Err(ProtocolError::InternalError(format!(
                    "Failed to cancel task: {}",
                    response.status()
                )))
            }
        }
        Err(e) => {
            error!("❌ A2A: HTTP request failed: {}", e);
            Err(ProtocolError::InternalError(format!(
                "Request failed: {}",
                e
            )))
        }
    }
}

pub async fn resubscribe_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Task>, ProtocolError> {
    info!("🔄 A2A: Resubscribing to task {}", task_id);

    let url = format!("{}/tasks/{}", state.agents_service_url, task_id);

    match state.http_client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        if let Some(task_data) = json_response.get("data") {
                            match serde_json::from_value::<Task>(task_data.clone()) {
                                Ok(task) => {
                                    info!("✅ A2A: Task {} resubscribed successfully", task_id);
                                    Ok(Json(task))
                                }
                                Err(e) => {
                                    error!("❌ A2A: Failed to parse task response: {}", e);
                                    Err(ProtocolError::InternalError(format!(
                                        "Failed to parse task: {}",
                                        e
                                    )))
                                }
                            }
                        } else {
                            error!("❌ A2A: No data field in response");
                            Err(ProtocolError::TaskNotFound(format!(
                                "Task {} not found",
                                task_id
                            )))
                        }
                    }
                    Err(e) => {
                        error!("❌ A2A: Failed to parse JSON response: {}", e);
                        Err(ProtocolError::InternalError(format!(
                            "Invalid response format: {}",
                            e
                        )))
                    }
                }
            } else if response.status() == 404 {
                Err(ProtocolError::TaskNotFound(format!(
                    "Task {} not found",
                    task_id
                )))
            } else {
                error!(
                    "❌ A2A: Task resubscribe failed with status {}",
                    response.status()
                );
                Err(ProtocolError::InternalError(format!(
                    "Failed to resubscribe to task: {}",
                    response.status()
                )))
            }
        }
        Err(e) => {
            error!("❌ A2A: HTTP request failed: {}", e);
            Err(ProtocolError::InternalError(format!(
                "Request failed: {}",
                e
            )))
        }
    }
}
