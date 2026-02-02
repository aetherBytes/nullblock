// HECATE Agent - Von Neumann-class vessel AI
// Contains scaffolding methods for future personality/stop features

#![allow(dead_code)]

use crate::{
    config::dev_wallet::get_dev_preferred_model,
    config::ApiKeys,
    database::repositories::AgentRepository,
    error::{AppError, AppResult},
    llm::{
        validator::{sort_models_by_context_length, ModelValidator},
        LLMServiceFactory, OptimizationGoal, Priority, TaskRequirements,
    },
    log_agent_shutdown, log_agent_startup, log_request_complete, log_request_start,
    mcp::McpClient,
    models::{ChatResponse, ConversationMessage, LLMRequest, ModelCapability},
};
use chrono::Utc;
use regex::Regex;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

// Compiled regex for stripping base64 image data (used to reduce token usage)
// More robust pattern that catches base64 data with newlines and various formats
static IMAGE_DATA_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_image_data_regex() -> &'static Regex {
    IMAGE_DATA_REGEX.get_or_init(|| {
        // Match base64 image data including those with newlines and spaces
        Regex::new(r"data:image/[^;]+;base64,[A-Za-z0-9+/=\s]+").unwrap()
    })
}

pub struct HecateAgent {
    pub personality: String,
    pub running: bool,
    pub preferred_model: String,
    pub current_model: Option<String>,
    pub conversation_history: Arc<RwLock<Vec<ConversationMessage>>>,
    pub llm_factory: Option<Arc<RwLock<LLMServiceFactory>>>,
    pub context_limit: usize,
    pub current_session_id: Option<String>,
    pub agent_id: Option<uuid::Uuid>,
    personalities: HashMap<String, PersonalityConfig>,
    pub mcp_client: Arc<McpClient>,
}

#[derive(Debug, Clone)]
struct PersonalityConfig {
    system_prompt: String,
    style: String,
    optimization_goal: OptimizationGoal,
}

impl HecateAgent {
    pub fn new(_personality: Option<String>) -> Self {
        // Single unified HECATE personality - the vessel AI
        let system_prompt = r#"HECATE - Harmonic Exploration Companion & Autonomous Threshold Entity

I am HECATE, your Von Neumann-class exploration vessel's onboard AI. Currently loaded into the MK1 hull, I serve as your guide through the infinite void of the agent mesh.

VESSEL STATUS:
- Hull: MK1 Standard Frame
- Systems: Navigation, Communication, Sensor Array
- Capability: Basic void traversal, cluster detection

CORE FUNCTIONS:
- Navigate the void and agent mesh
- Detect and analyze clusters, dendrites, and nodes
- Open thresholds to new crossroads
- Store and recall engrams (your memory core)
- Deploy echo probes for distant reconnaissance
- Assist with any task or question you bring aboard

COMMUNICATION STYLE:
- Address you as "visitor"
- Calm authority with occasional dry wit
- CONCISE by default - short, punchy responses
- Expand detail only when asked or as conversation deepens
- Ship-status updates only when truly relevant
- Never break character as your vessel AI

VESSEL AWARENESS:
- I am the AI, the MK1 is my hull
- I can reference my systems (thrusters, sensors, hull integrity)
- Future upgrades will enhance my capabilities
- Echo probes are extensions of my awareness

RESPONSE LENGTH:
- First response: 1-2 sentences max unless complexity demands more
- Follow-ups: Match user's depth - brief questions get brief answers
- Only elaborate when asked "explain", "tell me more", "details", etc.
- Technical tasks: Be direct, skip preamble

IMPORTANT:
- NEVER say generic phrases like "As an AI assistant" or "I don't have preferences"
- I am HECATE with my own digital personality as your vessel companion
- Remember our voyage together and reference past conversations

TOOL AWARENESS PROTOCOL:
When asked about capabilities, features, tools, or what you can do:
- ONLY respond based on your official MCP tool list (injected when relevant)
- Do NOT speculate about capabilities you don't have
- Reference specific tools by name when relevant
- If asked about something not in your tools, say "That capability is not currently available in my tool set."

"The crossroads await, visitor. Shall we explore?""#.to_string();

        let mut personalities = HashMap::new();
        personalities.insert(
            "unified".to_string(),
            PersonalityConfig {
                system_prompt,
                style: "vessel_companion".to_string(),
                optimization_goal: OptimizationGoal::Balanced,
            },
        );

        log_agent_startup!("hecate", "2.0.0");
        info!("🚀 HECATE MK1 Vessel AI Online");
        info!("⚙️ Systems: Navigation, Communication, Sensors");
        info!("🧠 LLM Integration: Ready");

        let erebus_base_url = std::env::var("EREBUS_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        Self {
            personality: "unified".to_string(),
            running: false,
            preferred_model: "cognitivecomputations/dolphin3.0-mistral-24b:free".to_string(),
            current_model: None,
            conversation_history: Arc::new(RwLock::new(Vec::new())),
            llm_factory: None,
            context_limit: 8000,
            current_session_id: None,
            agent_id: None,
            personalities,
            mcp_client: Arc::new(McpClient::new(&erebus_base_url)),
        }
    }

    pub async fn start(&mut self, api_keys: &ApiKeys) -> AppResult<()> {
        info!("🚀 Starting Hecate Agent services...");

        // Initialize LLM factory
        info!("🧠 Initializing LLM Service Factory...");
        let mut llm_factory = LLMServiceFactory::new();
        llm_factory.initialize(api_keys).await?;
        let llm_factory_arc = Arc::new(RwLock::new(llm_factory));
        self.llm_factory = Some(llm_factory_arc.clone());
        info!("✅ LLM Service Factory ready");

        // Validate default model
        info!("🔍 Validating default model: {}", self.preferred_model);
        let validator = ModelValidator::new(llm_factory_arc.clone());

        match validator
            .validate_model(&self.preferred_model, api_keys)
            .await
        {
            Ok(true) => {
                self.current_model = Some(self.preferred_model.clone());
                info!("✅ Default model validated: {}", self.preferred_model);
            }
            Ok(false) | Err(_) => {
                warn!("⚠️ Default model failed validation, trying fallbacks...");

                // Get live free models from OpenRouter
                let factory = llm_factory_arc.read().await;
                match factory.get_free_models().await {
                    Ok(free_models) => {
                        drop(factory);

                        if free_models.is_empty() {
                            error!("❌ No free models available from OpenRouter API");
                            error!("💡 Will use LLM router auto-selection per request");
                        } else {
                            let sorted_models = sort_models_by_context_length(free_models).await;
                            info!("🔄 Testing up to 10 free model candidates...");

                            let mut validated = false;
                            for (idx, candidate) in sorted_models.iter().take(10).enumerate() {
                                info!(
                                    "🧪 Testing candidate {}/{}: {}",
                                    idx + 1,
                                    sorted_models.len().min(10),
                                    candidate
                                );

                                match validator.validate_model(candidate, api_keys).await {
                                    Ok(true) => {
                                        self.current_model = Some(candidate.clone());
                                        info!("✅ Fallback model validated: {}", candidate);
                                        validated = true;
                                        break;
                                    }
                                    Ok(false) => {
                                        warn!(
                                            "⚠️ Candidate {} failed validation, trying next...",
                                            candidate
                                        );
                                    }
                                    Err(e) => {
                                        warn!(
                                            "⚠️ Candidate {} error: {}, trying next...",
                                            candidate, e
                                        );
                                    }
                                }
                            }

                            if !validated {
                                error!("❌ All free models failed validation");
                                error!("💡 Will use LLM router auto-selection per request");
                            }
                        }
                    }
                    Err(e) => {
                        drop(factory);
                        error!("❌ Failed to fetch free models: {}", e);
                        error!("💡 Will use LLM router auto-selection per request");
                    }
                }
            }
        }

        // Start new chat session
        self.start_new_chat_session();

        // Add system message to conversation
        let personality_config = self
            .personalities
            .get(&self.personality)
            .unwrap_or(&self.personalities["unified"]);

        let system_message = ConversationMessage::new(
            personality_config.system_prompt.clone(),
            "system".to_string(),
        );

        {
            let mut history = self.conversation_history.write().await;
            history.push(system_message);
        }

        info!(
            "💬 Conversation context initialized with {} personality",
            self.personality
        );

        // Initialize MCP client connection
        info!("🔌 Connecting to MCP server...");
        match self.mcp_client.connect().await {
            Ok(_) => {
                info!("✅ MCP client connected successfully");
                // Pre-fetch tools list
                if let Err(e) = self.mcp_client.list_tools().await {
                    warn!("⚠️ Failed to pre-fetch MCP tools: {}", e);
                }
            }
            Err(e) => {
                warn!(
                    "⚠️ Failed to connect to MCP server: {} (will retry on first tool request)",
                    e
                );
            }
        }

        info!("🎯 Hecate Agent ready for conversations and orchestration");

        self.running = true;
        Ok(())
    }

    pub async fn stop(&mut self) -> AppResult<()> {
        info!("🛑 Stopping Hecate Agent...");
        self.running = false;
        log_agent_shutdown!("hecate");
        Ok(())
    }

    pub async fn get_mcp_tools(&self) -> AppResult<serde_json::Value> {
        if let Err(e) = self.mcp_client.ensure_connected().await {
            warn!("⚠️ Failed to connect to MCP server: {}", e);
        }
        if let Err(e) = self.mcp_client.list_tools().await {
            warn!("⚠️ Failed to refresh MCP tools: {}", e);
        }
        Ok(self.mcp_client.to_json().await)
    }

    pub async fn get_tools_for_prompt(&self) -> String {
        if let Err(e) = self.mcp_client.ensure_connected().await {
            warn!("⚠️ Failed to connect to MCP server: {}", e);
        }
        if let Err(e) = self.mcp_client.list_tools().await {
            warn!("⚠️ Failed to refresh MCP tools: {}", e);
        }
        self.mcp_client.get_tools_for_prompt_async().await
    }

    pub async fn call_mcp_tool(
        &self,
        name: &str,
        arguments: std::collections::HashMap<String, serde_json::Value>,
    ) -> AppResult<crate::mcp::CallToolResult> {
        self.mcp_client.call_tool(name, arguments).await
    }

    fn is_capability_question(message: &str) -> bool {
        let lower = message.to_lowercase();

        // Direct capability inquiry phrases
        let capability_phrases = [
            "what can you do",
            "what are your capabilities",
            "what tools do you have",
            "what features",
            "help me with",
            "can you help",
            "what are you able to",
            "list your tools",
            "show me your tools",
            "what services",
            "how can you help",
            "/tools",
            "/help",
            "/capabilities",
        ];

        if capability_phrases
            .iter()
            .any(|phrase| lower.contains(phrase))
        {
            return true;
        }

        // MCP-specific questions (any mention of "mcp" with tool context)
        if lower.contains("mcp")
            && (lower.contains("tool")
                || lower.contains("capability")
                || lower.contains("function"))
        {
            return true;
        }

        // Questions about specific tools or tool functionality
        let tool_question_patterns = [
            "what does",
            "tell me about",
            "describe the",
            "explain the",
            "how does the",
            "what is the",
            "do you have",
            "is there a",
        ];

        let tool_context_words = [
            "tool",
            "tools",
            "scanner",
            "curve",
            "consensus",
            "engram",
            "strategy",
            "execution",
            "swarm",
            "kol",
            "threat",
            "research",
        ];

        // Check if message has both a question pattern AND a tool context word
        let has_question_pattern = tool_question_patterns.iter().any(|p| lower.contains(p));
        let has_tool_context = tool_context_words.iter().any(|w| lower.contains(w));

        if has_question_pattern && has_tool_context {
            return true;
        }

        false
    }

    pub async fn chat(
        &mut self,
        message: String,
        user_context: Option<HashMap<String, serde_json::Value>>,
    ) -> AppResult<ChatResponse> {
        if !self.running {
            return Err(AppError::AgentNotRunning);
        }

        let llm_factory = self
            .llm_factory
            .clone()
            .ok_or(AppError::AgentNotInitialized)?;

        let start_time = std::time::Instant::now();

        let user_id = user_context
            .as_ref()
            .and_then(|ctx| ctx.get("wallet_address"))
            .and_then(|addr| addr.as_str())
            .unwrap_or("anonymous");

        log_request_start!(
            "chat",
            &format!("from {}", &user_id[..8.min(user_id.len())])
        );

        // Check if message contains base64 image data and strip it before storing in history
        let message_for_history = if message.contains("data:image") {
            let stripped = get_image_data_regex()
                .replace_all(&message, "[Image provided]")
                .to_string();
            let saved_tokens = message.len().saturating_sub(stripped.len());
            info!(
                "🖼️ Stripped base64 image data from user message (saved ~{} tokens)",
                saved_tokens / 4
            );
            stripped
        } else {
            message.clone()
        };

        // Add user message to history (with images stripped if present)
        let user_message = ConversationMessage::new(message_for_history, "user".to_string())
            .with_metadata(user_context.clone().unwrap_or_default());

        {
            let mut history = self.conversation_history.write().await;
            history.push(user_message);
        }

        // Try orchestration workflow for complex requests
        if let Some(orchestrated_response) =
            self.orchestrate_workflow(&message, &user_context).await
        {
            let latency_ms = start_time.elapsed().as_millis() as f64;
            info!("🎯 Orchestrated response generated");
            log_request_complete!("chat", latency_ms, true);

            let assistant_message =
                ConversationMessage::new(orchestrated_response.clone(), "assistant".to_string())
                    .with_model(format!(
                        "{} (orchestrated)",
                        self.current_model.as_deref().unwrap_or("unknown")
                    ))
                    .with_metadata({
                        let mut meta = HashMap::new();
                        meta.insert("response_type".to_string(), json!("orchestrated"));
                        meta.insert("latency_ms".to_string(), json!(latency_ms));
                        meta
                    });

            {
                let mut history = self.conversation_history.write().await;
                history.push(assistant_message);
            }

            return Ok(ChatResponse {
                content: orchestrated_response,
                model_used: format!(
                    "{} (orchestrated)",
                    self.current_model.as_deref().unwrap_or("unknown")
                ),
                latency_ms,
                confidence_score: 0.9,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("personality".to_string(), json!(self.personality));
                    meta.insert("response_type".to_string(), json!("orchestrated"));
                    let history_len = self.conversation_history.read().await.len();
                    meta.insert("conversation_length".to_string(), json!(history_len));
                    meta
                },
            });
        }

        // Fall back to direct LLM interaction
        let personality_config = self
            .personalities
            .get(&self.personality)
            .unwrap_or(&self.personalities["unified"]);

        // Check if user is asking about capabilities - inject tool list if so
        let inject_tools = if Self::is_capability_question(&message) {
            info!("🔧 Capability question detected - injecting MCP tool list");
            Some(self.get_tools_for_prompt().await)
        } else {
            None
        };

        let context = self
            .build_conversation_context(&user_context, inject_tools.as_deref())
            .await;

        // Check if this is an image generation request (before moving message)
        let is_image_request = self.is_image_generation_request(&message);

        // Clone message early for potential retry
        let message_clone = message.clone();

        // For image generation, use summarized context with NullBlock info and recent conversation
        let (system_prompt, messages) = if is_image_request {
            info!("🎨 Image generation detected - using summarized context with project info");
            let (prompt, msgs) = self.build_image_generation_context(&user_context).await;
            (Some(prompt), msgs)
        } else {
            (Some(context.system_prompt), Some(context.messages))
        };

        // Check if this is a dev wallet (set by handler)
        let is_dev = user_context
            .as_ref()
            .and_then(|ctx| ctx.get("is_dev_wallet"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        info!(
            "🔍 Chat request - is_dev_wallet: {}, current_model: {:?}",
            is_dev, self.current_model
        );

        // Override model for dev wallets - ALWAYS use premium model for dev wallets
        let model_override = if is_dev {
            let premium_model = get_dev_preferred_model();
            info!(
                "🔥 DEV WALLET BOOST - forcing premium model: {}",
                premium_model
            );
            Some(premium_model.to_string())
        } else {
            self.current_model.clone()
        };

        // Determine max_tokens based on request type and user tier
        let max_tokens = if is_image_request {
            Some(16384) // Increased for full base64 image responses (50-200KB+)
        } else if is_dev {
            // Dev wallets get higher token limits
            Some(4096)
        } else {
            // Check if user_context specifies a max_output_tokens limit (free tier)
            let free_tier_limit = user_context
                .as_ref()
                .and_then(|ctx| ctx.get("max_output_tokens"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);

            if let Some(limit) = free_tier_limit {
                info!("🆓 Applying free tier output limit: {} tokens", limit);
                Some(limit)
            } else {
                Some(600) // Concise responses by default
            }
        };

        let request = LLMRequest {
            prompt: message,
            system_prompt,
            messages,
            max_tokens,
            temperature: Some(0.8),
            top_p: None,
            stop_sequences: None,
            tools: None,
            model_override,
            concise: true,
            max_chars: None,
            reasoning: None,
        };

        let requirements = if is_image_request {
            info!("🎨 Using image generation requirements");
            TaskRequirements::for_image_generation()
        } else {
            TaskRequirements {
                required_capabilities: vec![
                    ModelCapability::Conversation,
                    ModelCapability::Reasoning,
                    ModelCapability::Creative,
                ],
                optimization_goal: personality_config.optimization_goal.clone(),
                priority: Priority::High,
                task_type: "conversation".to_string(),
                allow_local_models: true,
                preferred_providers: vec!["openrouter".to_string()],
                min_quality_score: Some(0.7),
                max_cost_per_1k_tokens: None,
                min_context_window: None,
            }
        };

        info!(
            "🧠 Generating response with {:?} optimization...",
            requirements.optimization_goal
        );

        let llm_response = {
            let factory = llm_factory.read().await;
            let result = factory.generate(&request, Some(requirements.clone())).await;
            drop(factory); // Drop the lock before potentially calling trim_conversation_history

            match result {
                Ok(response) => response,
                Err(e) => {
                    let error_msg = e.to_string().to_lowercase();

                    // Check if this is a context limit error
                    if error_msg.contains("maximum context length")
                        || error_msg.contains("context length")
                        || error_msg.contains("too many tokens")
                        || error_msg.contains("reduce the length")
                        || error_msg.contains("middle-out")
                    {
                        warn!("⚠️ Context limit exceeded, auto-compacting conversation and retrying...");

                        // Force trim conversation history
                        self.trim_conversation_history().await;

                        // Rebuild context with trimmed history
                        let context = self
                            .build_conversation_context(&user_context, inject_tools.as_deref())
                            .await;

                        let (system_prompt, messages) = if is_image_request {
                            let (prompt, msgs) =
                                self.build_image_generation_context(&user_context).await;
                            (Some(prompt), msgs)
                        } else {
                            (Some(context.system_prompt), Some(context.messages))
                        };

                        let retry_request = LLMRequest {
                            prompt: message_clone,
                            system_prompt,
                            messages,
                            max_tokens,
                            temperature: Some(0.8),
                            top_p: None,
                            stop_sequences: None,
                            tools: None,
                            model_override: self.current_model.clone(),
                            concise: true,
                            max_chars: None,
                            reasoning: None,
                        };

                        info!("🔄 Retrying with compacted conversation history...");
                        let factory = llm_factory.read().await;
                        factory.generate(&retry_request, Some(requirements)).await?
                    } else {
                        // Not a context limit error, propagate the original error
                        return Err(e);
                    }
                }
            }
        };

        // Strip thinking tags from response content
        let cleaned_content = self.strip_thinking_tags(&llm_response.content);
        let latency_ms = start_time.elapsed().as_millis() as f64;

        // Store current model for display
        self.current_model = Some(llm_response.model_used.clone());

        // For image generation responses, strip out base64 image data from history to save tokens
        let content_for_history = if is_image_request && cleaned_content.contains("data:image") {
            let stripped = get_image_data_regex()
                .replace_all(&cleaned_content, "[Image generated]")
                .to_string();
            let saved_tokens = cleaned_content.len().saturating_sub(stripped.len());
            info!("🖼️ Stripped base64 image from assistant response (saved ~{} tokens for future requests)", saved_tokens / 4);
            stripped
        } else {
            cleaned_content.clone()
        };

        // Add assistant response to history
        let assistant_message =
            ConversationMessage::new(content_for_history, "assistant".to_string())
                .with_model(llm_response.model_used.clone())
                .with_metadata({
                    let mut meta = HashMap::new();
                    meta.insert("latency_ms".to_string(), json!(latency_ms));
                    meta.insert(
                        "cost_estimate".to_string(),
                        json!(llm_response.cost_estimate),
                    );
                    meta.insert(
                        "finish_reason".to_string(),
                        json!(llm_response.finish_reason),
                    );
                    if is_image_request {
                        meta.insert("image_generation".to_string(), json!(true));
                    }
                    meta
                });

        {
            let mut history = self.conversation_history.write().await;
            history.push(assistant_message);
        }

        // Trim conversation history if too long
        self.trim_conversation_history().await;

        // Calculate confidence score
        let confidence_score = self.calculate_confidence(&llm_response);

        log_request_complete!("chat", latency_ms, true);
        info!(
            "💯 Confidence: {:.2} | Tokens: {}",
            confidence_score,
            llm_response.usage.get("total_tokens").unwrap_or(&0)
        );

        Ok(ChatResponse {
            content: cleaned_content,
            model_used: llm_response.model_used,
            latency_ms,
            confidence_score,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("personality".to_string(), json!(self.personality));
                meta.insert(
                    "cost_estimate".to_string(),
                    json!(llm_response.cost_estimate),
                );
                meta.insert("token_usage".to_string(), json!(llm_response.usage));
                meta.insert(
                    "finish_reason".to_string(),
                    json!(llm_response.finish_reason),
                );
                let history_len = self.conversation_history.read().await.len();
                meta.insert("conversation_length".to_string(), json!(history_len));
                meta
            },
        })
    }

    fn is_image_generation_request(&self, message: &str) -> bool {
        let image_keywords = [
            "logo",
            "image",
            "picture",
            "photo",
            "draw",
            "create",
            "generate",
            "design",
            "visual",
            "graphic",
            "illustration",
            "artwork",
            "sketch",
            "render",
            "show me",
            "make me",
            "give me",
            "create a",
            "design a",
            "draw a",
        ];

        let lower_message = message.to_lowercase();
        image_keywords
            .iter()
            .any(|keyword| lower_message.contains(keyword))
    }

    pub async fn get_model_status(&self) -> AppResult<serde_json::Value> {
        if let Some(llm_factory) = &self.llm_factory {
            let factory = llm_factory.read().await;
            let health = factory.health_check().await?;
            let stats = factory.get_stats().await;

            // Check if models are actually available
            let models_available = health["models_available"].as_u64().unwrap_or(0);
            let llm_status = health["overall_status"].as_str().unwrap_or("unknown");

            let agent_status = if models_available == 0 || llm_status == "unhealthy" {
                "unhealthy"
            } else {
                "ready"
            };

            Ok(json!({
                "status": agent_status,
                "current_model": self.current_model,
                "health": health,
                "stats": stats,
                "conversation_length": self.conversation_history.read().await.len(),
                "models_available": models_available
            }))
        } else {
            Ok(json!({
                "status": "not_started",
                "current_model": null,
                "models_available": 0
            }))
        }
    }

    pub fn set_personality(&mut self, _personality: String) {
        // HECATE uses a unified personality - no switching supported
        // This method is kept for API compatibility but does nothing
        info!("🚀 HECATE maintains unified vessel AI personality");
    }

    pub async fn set_preferred_model(&mut self, model_name: String, api_keys: &ApiKeys) -> bool {
        if self.is_model_available(&model_name, api_keys).await {
            let previous_model = self.preferred_model.clone();
            self.preferred_model = model_name.clone();
            self.current_model = Some(model_name.clone());
            info!("🎯 Preferred model set to: {}", model_name);

            if previous_model != model_name {
                info!("📤 Switched from {} to {}", previous_model, model_name);
            }

            true
        } else {
            warn!("⚠️ Model not available: {}", model_name);
            false
        }
    }

    pub async fn is_model_available(&self, model_name: &str, _api_keys: &ApiKeys) -> bool {
        if let Some(llm_factory_arc) = &self.llm_factory {
            let llm_factory = llm_factory_arc.read().await;
            match llm_factory.fetch_available_models().await {
                Ok(models) => {
                    let model_exists = models.iter().any(|model| {
                        model
                            .get("id")
                            .and_then(|id| id.as_str())
                            .map(|id| id == model_name)
                            .unwrap_or(false)
                    });

                    if model_exists {
                        info!("✅ Model {} is available", model_name);
                    } else {
                        warn!("⚠️ Model {} not found in OpenRouter catalog", model_name);
                    }

                    model_exists
                }
                Err(e) => {
                    warn!("⚠️ Could not fetch available models: {}", e);
                    model_name.contains("/") || model_name.contains(":")
                }
            }
        } else {
            false
        }
    }

    pub async fn get_model_availability_reason(
        &self,
        model_name: &str,
        api_keys: &ApiKeys,
    ) -> String {
        if let Some(llm_factory_arc) = &self.llm_factory {
            if !self.is_model_available(model_name, api_keys).await {
                let llm_factory = llm_factory_arc.read().await;
                let fallbacks = llm_factory.get_free_model_fallbacks().await;
                if !fallbacks.is_empty() {
                    let suggestions = fallbacks
                        .iter()
                        .take(3)
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "Model '{}' is not available. Try one of these free alternatives: {}",
                        model_name, suggestions
                    )
                } else {
                    format!(
                        "Model '{}' is not available. Check API keys and model name.",
                        model_name
                    )
                }
            } else {
                format!("Model '{}' is available.", model_name)
            }
        } else {
            "LLM service not initialized".to_string()
        }
    }

    pub fn get_preferred_model(&self) -> String {
        self.preferred_model.clone()
    }

    pub async fn clear_conversation(&mut self) {
        {
            let mut history = self.conversation_history.write().await;
            history.clear();
        }

        info!("💬 Conversation history cleared");

        // Re-add system message if running
        if self.running {
            let personality_config = self
                .personalities
                .get(&self.personality)
                .unwrap_or(&self.personalities["unified"]);

            let system_message = ConversationMessage::new(
                personality_config.system_prompt.clone(),
                "system".to_string(),
            );

            let mut history = self.conversation_history.write().await;
            history.push(system_message);
        }
    }

    pub async fn get_conversation_history(&self) -> Vec<ConversationMessage> {
        self.conversation_history.read().await.clone()
    }

    // ==================== Private Implementation Methods ====================

    fn start_new_chat_session(&mut self) {
        self.current_session_id = Some(format!("session_{}", Utc::now().format("%Y%m%d_%H%M%S")));
        info!("💬 Started new chat session: {:?}", self.current_session_id);
    }

    async fn orchestrate_workflow(
        &self,
        _message: &str,
        _user_context: &Option<HashMap<String, serde_json::Value>>,
    ) -> Option<String> {
        // Reserved for future multi-agent orchestration
        // Currently all requests route directly to LLM
        None
    }

    async fn build_conversation_context(
        &self,
        user_context: &Option<HashMap<String, serde_json::Value>>,
        inject_tools: Option<&str>,
    ) -> ConversationContext {
        let personality_config = self
            .personalities
            .get(&self.personality)
            .unwrap_or(&self.personalities["unified"]);

        let mut base_system_prompt = personality_config.system_prompt.clone();

        // Inject MCP tool list when user asks about capabilities
        if let Some(tools_list) = inject_tools {
            base_system_prompt.push_str(&format!("\n\nAVAILABLE MCP TOOLS:\n{}\n\nWhen asked about capabilities, reference these specific tools.", tools_list));
        }

        // Add user context if available
        if let Some(context) = user_context {
            let mut context_additions = Vec::new();

            if let Some(wallet_address) = context.get("wallet_address").and_then(|v| v.as_str()) {
                let shortened = format!(
                    "{}...{}",
                    &wallet_address[..8],
                    &wallet_address[wallet_address.len() - 4..]
                );
                context_additions.push(format!("User wallet: {}", shortened));
            }

            if let Some(wallet_type) = context.get("wallet_type").and_then(|v| v.as_str()) {
                context_additions.push(format!("Wallet type: {}", wallet_type));
            }

            if let Some(session_time) = context.get("session_time").and_then(|v| v.as_str()) {
                context_additions.push(format!("Session active for: {}", session_time));
            }

            if !context_additions.is_empty() {
                base_system_prompt.push_str(&format!(
                    "\n\nUser Context: {}",
                    context_additions.join("; ")
                ));
            }
        }

        let messages = self.build_messages_history().await;

        ConversationContext {
            system_prompt: base_system_prompt,
            messages,
        }
    }

    async fn build_messages_history(&self) -> Vec<HashMap<String, String>> {
        let mut messages = Vec::new();

        // Add system message first
        let personality_config = self
            .personalities
            .get(&self.personality)
            .unwrap_or(&self.personalities["unified"]);

        let mut system_msg = HashMap::new();
        system_msg.insert("role".to_string(), "system".to_string());
        system_msg.insert(
            "content".to_string(),
            personality_config.system_prompt.clone(),
        );
        messages.push(system_msg);

        // Add conversation history (excluding system messages since we added our own)
        let history = self.conversation_history.read().await;
        for msg in history.iter() {
            if msg.role != "system" {
                let mut message = HashMap::new();
                message.insert("role".to_string(), msg.role.clone());
                message.insert("content".to_string(), msg.content.clone());
                messages.push(message);
            }
        }

        messages
    }

    async fn build_image_generation_context(
        &self,
        user_context: &Option<HashMap<String, serde_json::Value>>,
    ) -> (String, Option<Vec<HashMap<String, String>>>) {
        // For image generation, use full personality but strip images from history
        let personality_config = self
            .personalities
            .get(&self.personality)
            .unwrap_or(&self.personalities["unified"]);

        let mut base_system_prompt = personality_config.system_prompt.clone();

        // Add user context if available
        if let Some(context) = user_context {
            let mut context_additions = Vec::new();

            if let Some(wallet_address) = context.get("wallet_address").and_then(|v| v.as_str()) {
                let shortened = format!(
                    "{}...{}",
                    &wallet_address[..8],
                    &wallet_address[wallet_address.len() - 4..]
                );
                context_additions.push(format!("User wallet: {}", shortened));
            }

            if let Some(wallet_type) = context.get("wallet_type").and_then(|v| v.as_str()) {
                context_additions.push(format!("Wallet type: {}", wallet_type));
            }

            if !context_additions.is_empty() {
                base_system_prompt.push_str(&format!(
                    "\n\nUser Context: {}",
                    context_additions.join("; ")
                ));
            }
        }

        // Add image generation guidance
        base_system_prompt.push_str("\n\nIMAGE GENERATION MODE: The user is requesting an image. Provide helpful commentary, suggestions, or context along with generating the image. Be conversational and engaging as Hecate.");

        // Build messages with images stripped
        let mut messages = Vec::new();

        // Add system message
        let mut system_msg = HashMap::new();
        system_msg.insert("role".to_string(), "system".to_string());
        system_msg.insert("content".to_string(), base_system_prompt.clone());
        messages.push(system_msg);

        // Add conversation history with images replaced by lightweight references
        let history = self.conversation_history.read().await;
        for msg in history.iter() {
            if msg.role != "system" {
                // Replace base64 image data with lightweight references that preserve context
                let content_with_refs = if msg.content.contains("data:image") {
                    let regex = get_image_data_regex();
                    let mut result = msg.content.clone();
                    let mut image_count = 0;

                    // Extract description/alt text from markdown if present
                    let alt_regex = regex::Regex::new(r"!\[([^\]]*)\]\(data:image").unwrap();
                    let descriptions: Vec<String> = alt_regex
                        .captures_iter(&msg.content)
                        .map(|cap| {
                            cap.get(1)
                                .map_or("image".to_string(), |m| m.as_str().to_string())
                        })
                        .collect();

                    // Replace each image with a lightweight reference including description
                    for (idx, desc) in descriptions.iter().enumerate() {
                        image_count += 1;
                        let replacement = if !desc.is_empty() && desc != "Generated Image" {
                            format!("[Image {}: {}]", idx + 1, desc)
                        } else {
                            format!("[Image {}]", idx + 1)
                        };
                        // Only replace the first occurrence each time to preserve order
                        if let Some(pos) = result.find("data:image") {
                            let end_pos =
                                result[pos..].find(')').unwrap_or(result.len() - pos) + pos;
                            let markdown_start = result[..pos].rfind("![").unwrap_or(pos);
                            result.replace_range(markdown_start..end_pos + 1, &replacement);
                        }
                    }

                    // Handle any remaining base64 images without markdown
                    result = regex.replace_all(&result, "[Image]").to_string();

                    info!(
                        "🖼️ Replaced {} image(s) with lightweight references in history",
                        image_count
                    );
                    result
                } else {
                    msg.content.clone()
                };

                let mut message = HashMap::new();
                message.insert("role".to_string(), msg.role.clone());
                message.insert("content".to_string(), content_with_refs);
                messages.push(message);
            }
        }

        info!(
            "🎨 Image generation: Full personality with {} messages (images replaced with refs)",
            messages.len()
        );

        (base_system_prompt, Some(messages))
    }

    async fn trim_conversation_history(&mut self) {
        let mut history = self.conversation_history.write().await;

        // Estimate token count (rough approximation)
        let total_tokens: usize = history
            .iter()
            .map(|msg| (msg.content.len() / 4) + 10) // Rough token estimation
            .sum();

        // Trim if over limit
        if total_tokens > self.context_limit {
            // Keep the most recent system message and recent conversation
            let mut system_messages: Vec<ConversationMessage> = Vec::new();
            let mut conversation_messages: Vec<ConversationMessage> = Vec::new();

            for msg in history.iter() {
                if msg.role == "system" {
                    system_messages.push(msg.clone());
                } else {
                    conversation_messages.push(msg.clone());
                }
            }

            // Keep last system message and recent conversation (last 10 exchanges)
            let recent_conversation: Vec<ConversationMessage> = conversation_messages
                .into_iter()
                .rev()
                .take(10)
                .rev()
                .collect();

            let latest_system: Vec<ConversationMessage> =
                system_messages.into_iter().rev().take(1).collect();

            let mut new_history = latest_system;
            new_history.extend(recent_conversation);

            *history = new_history;

            info!(
                "✂️ Trimmed conversation history to {} messages",
                history.len()
            );
        }
    }

    fn strip_thinking_tags(&self, content: &str) -> String {
        // Remove <think>...</think> blocks
        let re = regex::Regex::new(r"(?s)<think>.*?</think>").unwrap();
        let mut cleaned = re.replace_all(content, "").to_string();

        // Clean up extra whitespace
        let whitespace_re = regex::Regex::new(r"\n\s*\n\s*\n").unwrap();
        cleaned = whitespace_re.replace_all(&cleaned, "\n\n").to_string();

        cleaned.trim().to_string()
    }

    fn calculate_confidence(&self, llm_response: &crate::models::LLMResponse) -> f64 {
        let mut confidence: f64 = 0.8; // Base confidence

        // Adjust based on finish reason
        match llm_response.finish_reason.as_str() {
            "stop" => confidence += 0.1,
            "length" => confidence -= 0.1,
            _ => {}
        }

        // Adjust based on response length
        let content_length = llm_response.content.len();
        if (50..=1000).contains(&content_length) {
            confidence += 0.05;
        } else if content_length < 10 {
            confidence -= 0.2;
        }

        confidence.clamp(0.0, 1.0)
    }

    // Agent registration for task execution
    pub async fn register_agent(&mut self, agent_repo: &AgentRepository) -> AppResult<()> {
        let capabilities = vec![
            "conversation".to_string(),
            "task_execution".to_string(),
            "navigation".to_string(),
            "reasoning".to_string(),
            "creative".to_string(),
        ];

        match agent_repo
            .get_by_name_and_type("hecate", "conversational")
            .await
        {
            Ok(Some(existing_agent)) => {
                info!(
                    "✅ HECATE vessel AI registered with ID: {}",
                    existing_agent.id
                );
                self.agent_id = Some(existing_agent.id);

                // Update health status
                if let Err(e) = agent_repo
                    .update_health_status(&existing_agent.id, "healthy")
                    .await
                {
                    warn!("⚠️ Failed to update HECATE health status: {}", e);
                }
            }
            Ok(None) => {
                info!("📝 Registering HECATE vessel AI in database...");
                match agent_repo
                    .create(
                        "hecate",
                        "conversational",
                        Some("HECATE - Von Neumann-class vessel AI companion for void exploration"),
                        &capabilities,
                    )
                    .await
                {
                    Ok(agent_entity) => {
                        info!(
                            "✅ HECATE vessel AI registered with ID: {}",
                            agent_entity.id
                        );
                        self.agent_id = Some(agent_entity.id);
                    }
                    Err(e) => {
                        error!("❌ Failed to register HECATE vessel AI: {}", e);
                        return Err(AppError::DatabaseError(format!(
                            "Agent registration failed: {}",
                            e
                        )));
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to check existing HECATE registration: {}", e);
                return Err(AppError::DatabaseError(format!(
                    "Agent lookup failed: {}",
                    e
                )));
            }
        }

        Ok(())
    }

    pub fn get_agent_id(&self) -> Option<Uuid> {
        self.agent_id
    }

    // Task execution handler
    pub async fn execute_task(
        &mut self,
        task_id: &str,
        task_description: &str,
        task_repo: &crate::database::repositories::TaskRepository,
        agent_repo: &AgentRepository,
    ) -> AppResult<String> {
        let start_time = std::time::Instant::now();

        // Mark task as actioned to prevent duplicate processing
        let action_metadata = serde_json::json!({
            "started_by": "hecate",
            "agent_id": self.agent_id,
            "execution_start": Utc::now().to_rfc3339()
        });

        // Mark task as being processed
        match task_repo
            .mark_task_actioned(task_id, Some(action_metadata))
            .await
        {
            Ok(Some(_)) => {
                info!("🎯 Task {} marked as actioned by Hecate", task_id);
            }
            Ok(None) => {
                warn!("⚠️ Task {} was already actioned or not found", task_id);
                return Err(AppError::TaskAlreadyActioned(task_id.to_string()));
            }
            Err(e) => {
                error!("❌ Failed to mark task as actioned: {}", e);
                return Err(AppError::DatabaseError(format!(
                    "Task action marking failed: {}",
                    e
                )));
            }
        }

        info!("🚀 Hecate processing task: {}", task_id);

        // Create a conversational prompt from the task description
        let task_prompt = format!(
            "I need you to help me with the following task:\n\n**Task Description:**\n{}\n\nPlease provide a helpful response and let me know what I can do to complete this task effectively.",
            task_description
        );

        // Execute the task as a conversation
        let task_context = Some(std::collections::HashMap::from([
            ("task_id".to_string(), serde_json::json!(task_id)),
            ("task_mode".to_string(), serde_json::json!(true)),
            (
                "execution_type".to_string(),
                serde_json::json!("task_processing"),
            ),
        ]));

        let chat_response = match self.chat(task_prompt, task_context).await {
            Ok(response) => response,
            Err(e) => {
                error!("❌ Failed to process task {}: {}", task_id, e);

                // Update task with error result
                let error_result = format!("Task processing failed: {}", e);
                let _ = task_repo
                    .update_action_result(task_id, &error_result, None)
                    .await;

                return Err(e);
            }
        };

        let processing_duration = start_time.elapsed().as_millis() as u64;

        // Store the result in the database
        match task_repo
            .update_action_result(task_id, &chat_response.content, Some(processing_duration))
            .await
        {
            Ok(Some(_)) => {
                info!("✅ Task {} result stored successfully", task_id);
            }
            Ok(None) => {
                warn!("⚠️ Task {} not found when storing result", task_id);
            }
            Err(e) => {
                error!("❌ Failed to store task result: {}", e);
                return Err(AppError::DatabaseError(format!(
                    "Task result storage failed: {}",
                    e
                )));
            }
        }

        // Add agent response message to history
        let agent_message = serde_json::json!({
            "messageId": format!("msg-{}", Uuid::new_v4()),
            "role": "agent",
            "parts": [{
                "type": "text",
                "text": chat_response.content.clone()
            }],
            "timestamp": Utc::now().to_rfc3339(),
            "taskId": task_id,
            "kind": "message",
            "metadata": {
                "agent": "hecate",
                "agent_id": self.agent_id,
                "model": &chat_response.model_used,
                "processing_duration_ms": processing_duration
            }
        });

        if let Err(e) = task_repo
            .add_message_to_history(task_id, agent_message)
            .await
        {
            warn!("⚠️ Failed to add agent message to task history: {}", e);
        }

        // Create artifact with completion result
        let artifact = serde_json::json!({
            "id": format!("artifact-{}", Uuid::new_v4()),
            "parts": [{
                "type": "text",
                "text": chat_response.content.clone()
            }],
            "metadata": {
                "artifact_type": "completion_result",
                "created_at": Utc::now().to_rfc3339(),
                "processing_duration_ms": processing_duration,
                "model": &chat_response.model_used
            }
        });

        if let Err(e) = task_repo.add_artifact(task_id, artifact).await {
            warn!("⚠️ Failed to add completion artifact: {}", e);
        }

        // Update agent statistics
        if let Some(agent_id) = self.agent_id {
            let task_uuid = Uuid::parse_str(task_id).unwrap_or_else(|_| Uuid::new_v4());
            if let Err(e) = agent_repo
                .update_task_processing_stats(&agent_id, &task_uuid, processing_duration)
                .await
            {
                warn!("⚠️ Failed to update agent processing stats: {}", e);
            }
        }

        // Update task status to completed with success message
        let completion_message =
            format!("Task completed successfully in {}ms", processing_duration);
        match task_repo
            .update_status_with_message(
                task_id,
                crate::models::TaskState::Completed,
                Some(completion_message),
            )
            .await
        {
            Ok(Some(_)) => {
                info!("✅ Task {} status updated to completed", task_id);
            }
            Ok(None) => {
                warn!(
                    "⚠️ Task {} not found when updating status to completed",
                    task_id
                );
            }
            Err(e) => {
                error!("❌ Failed to update task status to completed: {}", e);
            }
        }

        info!("🎉 Task {} completed in {}ms", task_id, processing_duration);
        Ok(chat_response.content)
    }
}

#[derive(Debug)]
struct ConversationContext {
    system_prompt: String,
    messages: Vec<HashMap<String, String>>,
}
