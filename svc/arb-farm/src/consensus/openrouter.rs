use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::error::{AppError, AppResult};

pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenRouterClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
        }
    }

    pub async fn query_model(
        &self,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
        max_tokens: u32,
    ) -> AppResult<ModelResponse> {
        let start = Instant::now();

        let messages = if let Some(sys) = system_prompt {
            vec![
                ChatMessage::text("system", sys),
                ChatMessage::text("user", prompt),
            ]
        } else {
            vec![ChatMessage::text("user", prompt)]
        };

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            max_tokens: Some(max_tokens),
            temperature: Some(0.3),
            tools: None,
            tool_choice: None,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://nullblock.io")
            .header("X-Title", "ArbFarm Consensus")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("OpenRouter request failed: {}", e)))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ExternalApi(format!(
                "OpenRouter API error ({}): {}",
                status, body
            )));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse response: {}", e)))?;

        let content = chat_response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(ModelResponse {
            model: model.to_string(),
            content,
            latency_ms,
            tokens_used: chat_response.usage.map(|u| u.total_tokens).unwrap_or(0),
        })
    }

    pub async fn query_model_with_tools(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        tools: &[ToolDefinition],
        max_tokens: u32,
    ) -> AppResult<ToolModelResponse> {
        let start = Instant::now();

        let tools_param = if tools.is_empty() {
            None
        } else {
            Some(tools.to_vec())
        };

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            max_tokens: Some(max_tokens),
            temperature: Some(0.3),
            tools: tools_param,
            tool_choice: if tools.is_empty() {
                None
            } else {
                Some(ToolChoice::Auto)
            },
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://nullblock.io")
            .header("X-Title", "ArbFarm Consensus")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("OpenRouter request failed: {}", e)))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ExternalApi(format!(
                "OpenRouter API error ({}): {}",
                status, body
            )));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse response: {}", e)))?;

        let choice = chat_response
            .choices
            .first()
            .ok_or_else(|| AppError::ExternalApi("OpenRouter returned no choices".to_string()))?;

        let response_type = if let Some(ref tool_calls) = choice.message.tool_calls {
            if !tool_calls.is_empty() {
                let calls: Vec<ToolCall> = tool_calls
                    .iter()
                    .filter_map(|tc| {
                        let args: serde_json::Value =
                            serde_json::from_str(&tc.function.arguments).ok()?;
                        Some(ToolCall {
                            id: tc.id.clone(),
                            name: tc.function.name.clone(),
                            arguments: args,
                        })
                    })
                    .collect();
                ToolModelResponseType::ToolUse(calls)
            } else {
                ToolModelResponseType::Text(choice.message.content.clone().unwrap_or_default())
            }
        } else {
            ToolModelResponseType::Text(choice.message.content.clone().unwrap_or_default())
        };

        Ok(ToolModelResponse {
            model: model.to_string(),
            response: response_type,
            latency_ms,
            tokens_used: chat_response.usage.map(|u| u.total_tokens).unwrap_or(0),
        })
    }

    pub async fn query_structured<T: for<'de> Deserialize<'de>>(
        &self,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> AppResult<(T, u64)> {
        let response = self.query_model(model, prompt, system_prompt, 2048).await?;

        let json_start = response.content.find('{');
        let json_end = response.content.rfind('}');

        let json_str = match (json_start, json_end) {
            (Some(start), Some(end)) => &response.content[start..=end],
            _ => {
                return Err(AppError::ExternalApi(
                    "No JSON found in model response".to_string(),
                ))
            }
        };

        let parsed: T = serde_json::from_str(json_str)
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse JSON response: {}", e)))?;

        Ok((parsed, response.latency_ms))
    }
}

#[derive(Debug, Clone, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<RawToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn text(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant_with_tool_calls(tool_calls: Vec<RawToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }

    pub fn tool_result(tool_call_id: &str, result: &str) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(result.to_string()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: RawFunctionCall,
}

impl RawToolCall {
    pub fn new(id: &str, name: &str, arguments: &serde_json::Value) -> Self {
        Self {
            id: id.to_string(),
            call_type: "function".to_string(),
            function: RawFunctionCall {
                name: name.to_string(),
                arguments: arguments.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
struct Choice {
    message: ChatMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Usage {
    total_tokens: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

impl ToolDefinition {
    pub fn from_mcp_tool(name: &str, description: &str, input_schema: serde_json::Value) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: name.to_string(),
                description: Some(description.to_string()),
                parameters: input_schema,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoice {
    Auto,
    None,
    #[serde(rename = "required")]
    Required,
}

#[derive(Debug, Clone)]
pub struct ModelResponse {
    pub model: String,
    pub content: String,
    pub latency_ms: u64,
    pub tokens_used: u32,
}

#[derive(Debug, Clone)]
pub struct ToolModelResponse {
    pub model: String,
    pub response: ToolModelResponseType,
    pub latency_ms: u64,
    pub tokens_used: u32,
}

#[derive(Debug, Clone)]
pub enum ToolModelResponseType {
    Text(String),
    ToolUse(Vec<ToolCall>),
}

impl ToolModelResponse {
    pub fn is_tool_use(&self) -> bool {
        matches!(self.response, ToolModelResponseType::ToolUse(_))
    }

    pub fn text_content(&self) -> Option<&str> {
        match &self.response {
            ToolModelResponseType::Text(t) => Some(t),
            _ => None,
        }
    }

    pub fn tool_calls(&self) -> Option<&Vec<ToolCall>> {
        match &self.response {
            ToolModelResponseType::ToolUse(calls) => Some(calls),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

pub const AVAILABLE_MODELS: &[(&str, &str, f64)] = &[
    ("anthropic/claude-3.5-sonnet", "Claude 3.5 Sonnet", 1.5),
    ("openai/gpt-4-turbo", "GPT-4 Turbo", 1.0),
    ("meta-llama/llama-3.1-70b-instruct", "Llama 3.1 70B", 0.8),
    ("google/gemini-pro-1.5", "Gemini Pro 1.5", 0.9),
    ("mistralai/mistral-large", "Mistral Large", 0.7),
];

pub fn get_default_models() -> Vec<String> {
    vec![
        "anthropic/claude-3.5-sonnet".to_string(),
        "openai/gpt-4-turbo".to_string(),
        "meta-llama/llama-3.1-70b-instruct".to_string(),
    ]
}

pub fn get_model_weight(model: &str) -> f64 {
    AVAILABLE_MODELS
        .iter()
        .find(|(id, _, _)| *id == model)
        .map(|(_, _, weight)| *weight)
        .unwrap_or(1.0)
}

pub fn mcp_tools_to_openrouter(tools: &[nullblock_mcp_client::McpTool]) -> Vec<ToolDefinition> {
    tools
        .iter()
        .map(|t| {
            ToolDefinition::from_mcp_tool(
                &t.name,
                t.description.as_deref().unwrap_or(""),
                t.input_schema.clone(),
            )
        })
        .collect()
}

pub async fn quick_llm_call(prompt: &str) -> Result<String, String> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY not configured".to_string())?;

    let client = OpenRouterClient::new(api_key);

    let model = "anthropic/claude-3.5-sonnet";

    match client.query_model(model, prompt, None, 2000).await {
        Ok(response) => Ok(response.content),
        Err(e) => Err(format!("LLM call failed: {}", e)),
    }
}
