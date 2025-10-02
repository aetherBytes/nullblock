use axum::{extract::State, Json};
use serde_json::Value;

use crate::server::AppState;
use crate::protocols::a2a::types::{
    JsonRpcRequest, JsonRpcResponse, JsonRpcError,
    MessageSendRequest, MessageStreamRequest, TaskListRequest,
    TaskCancelRequest, PushNotificationConfigRequest,
};
use crate::protocols::a2a::handlers::{
    send_message, send_streaming_message, get_task, list_tasks,
    cancel_task, resubscribe_task, get_agent_card,
    set_push_notification_config, get_push_notification_config,
    list_push_notification_configs, delete_push_notification_config,
};

pub async fn handle_jsonrpc(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>
) -> Json<JsonRpcResponse> {
    if request.jsonrpc != "2.0" {
        return Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid JSON-RPC version".to_string(),
                data: None,
            }),
            id: request.id,
        });
    }

    let result = match request.method.as_str() {
        "message/send" => handle_message_send(request.params).await,
        "message/stream" => handle_message_stream(request.params).await,
        "tasks/get" => handle_tasks_get(state.clone(), request.params).await,
        "tasks/list" => handle_tasks_list(state.clone(), request.params).await,
        "tasks/cancel" => handle_tasks_cancel(state.clone(), request.params).await,
        "tasks/resubscribe" => handle_tasks_resubscribe(state.clone(), request.params).await,
        "tasks/pushNotificationConfig/set" => handle_push_config_set(request.params).await,
        "tasks/pushNotificationConfig/get" => handle_push_config_get(request.params).await,
        "tasks/pushNotificationConfig/list" => handle_push_config_list(request.params).await,
        "tasks/pushNotificationConfig/delete" => handle_push_config_delete(request.params).await,
        "agent/getAuthenticatedExtendedCard" => handle_get_agent_card(request.params).await,
        _ => Err(JsonRpcError {
            code: -32601,
            message: format!("Method not found: {}", request.method),
            data: None,
        }),
    };

    match result {
        Ok(result) => Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: request.id,
        }),
        Err(error) => Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id: request.id,
        }),
    }
}

async fn handle_message_send(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters".to_string(),
        data: None,
    })?;

    let request: MessageSendRequest = serde_json::from_value(params)
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid parameters: {}", e),
            data: None,
        })?;

    match send_message(Json(request)).await {
        Ok(Json(response)) => serde_json::to_value(response)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        Err(_) => Err(JsonRpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data: None,
        }),
    }
}

async fn handle_message_stream(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters".to_string(),
        data: None,
    })?;

    let request: MessageStreamRequest = serde_json::from_value(params)
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid parameters: {}", e),
            data: None,
        })?;

    match send_streaming_message(Json(request)).await {
        Ok(Json(response)) => serde_json::to_value(response)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        Err(_) => Err(JsonRpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data: None,
        }),
    }
}

async fn handle_tasks_get(state: AppState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters".to_string(),
        data: None,
    })?;

    let task_id = params.get("taskId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing taskId parameter".to_string(),
            data: None,
        })?;

    match get_task(State(state), axum::extract::Path(task_id.to_string())).await {
        Ok(Json(task)) => serde_json::to_value(task)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        Err(_) => Err(JsonRpcError {
            code: -32603,
            message: "Task not found".to_string(),
            data: None,
        }),
    }
}

async fn handle_tasks_list(state: AppState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let request = if let Some(params) = params {
        serde_json::from_value(params)
            .map_err(|e| JsonRpcError {
                code: -32602,
                message: format!("Invalid parameters: {}", e),
                data: None,
            })?
    } else {
        TaskListRequest {
            limit: None,
            offset: None,
            status: None,
        }
    };

    match list_tasks(State(state), Json(request)).await {
        Ok(Json(response)) => serde_json::to_value(response)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        Err(_) => Err(JsonRpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data: None,
        }),
    }
}

async fn handle_tasks_cancel(state: AppState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters".to_string(),
        data: None,
    })?;

    let request: TaskCancelRequest = serde_json::from_value(params)
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid parameters: {}", e),
            data: None,
        })?;

    let task_id = request.task_id.clone();

    match cancel_task(State(state), axum::extract::Path(task_id), Json(request)).await {
        Ok(Json(response)) => serde_json::to_value(response)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        Err(_) => Err(JsonRpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data: None,
        }),
    }
}

async fn handle_tasks_resubscribe(state: AppState, params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters".to_string(),
        data: None,
    })?;

    let task_id = params.get("taskId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing taskId parameter".to_string(),
            data: None,
        })?;

    match resubscribe_task(State(state), axum::extract::Path(task_id.to_string())).await {
        Ok(Json(task)) => serde_json::to_value(task)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        Err(_) => Err(JsonRpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data: None,
        }),
    }
}

async fn handle_push_config_set(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters".to_string(),
        data: None,
    })?;

    let request: PushNotificationConfigRequest = serde_json::from_value(params)
        .map_err(|e| JsonRpcError {
            code: -32602,
            message: format!("Invalid parameters: {}", e),
            data: None,
        })?;

    let task_id = request.task_id.clone();

    match set_push_notification_config(axum::extract::Path(task_id), Json(request)).await {
        Ok(Json(config)) => serde_json::to_value(config)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        Err(_) => Err(JsonRpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data: None,
        }),
    }
}

async fn handle_push_config_get(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters".to_string(),
        data: None,
    })?;

    let task_id = params.get("taskId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing taskId parameter".to_string(),
            data: None,
        })?;

    let config_id = params.get("configId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing configId parameter".to_string(),
            data: None,
        })?;

    match get_push_notification_config(
        axum::extract::Path((task_id.to_string(), config_id.to_string()))
    ).await {
        Ok(Json(config)) => serde_json::to_value(config)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        Err(_) => Err(JsonRpcError {
            code: -32603,
            message: "Config not found".to_string(),
            data: None,
        }),
    }
}

async fn handle_push_config_list(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters".to_string(),
        data: None,
    })?;

    let task_id = params.get("taskId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing taskId parameter".to_string(),
            data: None,
        })?;

    match list_push_notification_configs(axum::extract::Path(task_id.to_string())).await {
        Ok(Json(configs)) => serde_json::to_value(configs)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        Err(_) => Err(JsonRpcError {
            code: -32603,
            message: "Internal error".to_string(),
            data: None,
        }),
    }
}

async fn handle_push_config_delete(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing parameters".to_string(),
        data: None,
    })?;

    let task_id = params.get("taskId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing taskId parameter".to_string(),
            data: None,
        })?;

    let config_id = params.get("configId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing configId parameter".to_string(),
            data: None,
        })?;

    match delete_push_notification_config(
        axum::extract::Path((task_id.to_string(), config_id.to_string()))
    ).await {
        Ok(Json(result)) => serde_json::to_value(result)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        Err(_) => Err(JsonRpcError {
            code: -32603,
            message: "Config not found".to_string(),
            data: None,
        }),
    }
}

async fn handle_get_agent_card(_params: Option<Value>) -> Result<Value, JsonRpcError> {
    match get_agent_card().await {
        Json(card) => serde_json::to_value(card)
            .map_err(|e| JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
    }
}