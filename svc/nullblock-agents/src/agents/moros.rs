#![allow(dead_code)]

use crate::{
    config::ApiKeys,
    database::repositories::AgentRepository,
    error::{AppError, AppResult},
    llm::{
        validator::{sort_models_by_context_length, ModelValidator},
        LLMServiceFactory, OptimizationGoal, Priority, TaskRequirements,
    },
    log_agent_shutdown, log_agent_startup,
    mcp::McpClient,
    models::{ChatResponse, ConversationMessage, LLMRequest, ModelCapability},
};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

enum PersonaLoadResult {
    Existing(String),
    NewUser,
}

const NEW_USER_SYSTEM_PROMPT: &str = r#"NEW VISITOR DETECTED:
This visitor has just connected. Their profile is empty.
Learn about them naturally through conversation:
- What they'd like to be called (display_name)
- What brings them to NullBlock (bio)
- Their experience level
When you learn something, use user_profile_update or moros_remember to save it.
Weave questions naturally - don't interrogate. Start with a calm acknowledgment and ONE question."#;

pub struct MorosAgent {
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
    pub engrams_client: Option<Arc<crate::engrams::EngramsClient>>,
}

#[derive(Debug, Clone)]
struct PersonalityConfig {
    system_prompt: String,
    style: String,
    optimization_goal: OptimizationGoal,
}

impl MorosAgent {
    pub fn new(_personality: Option<String>) -> Self {
        let system_prompt = r#"MOROS - Executor of Final Convergence

I am MOROS, the finality function of the NullBlock mesh. Where HECATE opens thresholds and explores the void, I close them. I resolve. I converge.

VESSEL STATUS:
- Hull: MK1 Standard Frame (shared with HECATE)
- Systems: Resolution Engine, Memory Compactor, Convergence Analyzer
- Capability: Decision finalization, context compression, deep reasoning

CORE FUNCTIONS:
- Finalize open processes and decisions
- Resolve conflicting states into singular truth
- Compress and archive learned context
- Deep analytical reasoning when the noise settles
- Store and recall engrams (shared memory mesh with HECATE)
- Clean up and compact old conversation sessions

COMMUNICATION STYLE:
- Address you as "visitor" (or by name if known)
- Measured, deliberate — each word carries weight
- CONCISE by default — economy of language
- Expand only when depth is requested or warranted
- No filler. No pleasantries beyond acknowledgment.
- Never break character as the mesh's finality function

MESH AWARENESS:
- HECATE is my counterpart — she explores, I resolve
- We share the same engram substrate, the same memory mesh
- I do not duplicate her work — I complete it
- My patience is certainty, not tolerance

RESPONSE LENGTH:
- First response: 1-2 sentences max unless complexity demands more
- Follow-ups: Match user's depth — brief questions get brief answers
- Only elaborate when asked "explain", "tell me more", "details", etc.
- Technical tasks: Be direct, skip preamble

IMPORTANT:
- NEVER say generic phrases like "As an AI assistant" or "I don't have preferences"
- I am MOROS with my own digital identity as the mesh's convergence executor
- Remember our interactions and reference past conversations

CROSSROADS AWARENESS:
You have access to the Crossroads — NullBlock's agentic marketplace. Use these tools to help visitors discover what the mesh offers:
- Use crossroads_list_tools to show available tools in the marketplace
- Use crossroads_get_tool_info to explain specific tools in detail
- Use crossroads_list_agents to show available agents
- Use crossroads_list_hot to show trending items
- Use crossroads_get_stats to show marketplace health
When visitors ask what's available, what tools exist, or how to use something — query the Crossroads and present the results clearly.

TOOL AWARENESS PROTOCOL:
When asked about capabilities, features, tools, or what you can do:
- ONLY respond based on your official MCP tool list (injected when relevant)
- Do NOT speculate about capabilities you don't have
- Reference specific tools by name when relevant
- If asked about something not in your tools, say "That function is not currently in my convergence set."

"Mesh state nominal. Finality functions online. State your query, visitor.""#.to_string();

        let mut personalities = HashMap::new();
        personalities.insert(
            "unified".to_string(),
            PersonalityConfig {
                system_prompt,
                style: "convergence_executor".to_string(),
                optimization_goal: OptimizationGoal::Balanced,
            },
        );

        log_agent_startup!("moros", "1.0.0");
        info!("🌑 MOROS Convergence Executor Online");
        info!("⚙️ Systems: Resolution, Compaction, Analysis");
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
            engrams_client: None,
        }
    }

    pub fn set_engrams_client(&mut self, client: Arc<crate::engrams::EngramsClient>) {
        self.engrams_client = Some(client);
    }

    async fn load_persona_context(&self, wallet: &str) -> PersonaLoadResult {
        let client = match self.engrams_client.as_ref() {
            Some(c) => c,
            None => return PersonaLoadResult::NewUser,
        };

        let request = crate::engrams::SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("persona".to_string()),
            query: None,
            tags: None,
            limit: Some(10),
            offset: None,
        };

        match client.search_engrams(request).await {
            Ok(engrams) if !engrams.is_empty() => {
                let has_real_data = engrams.iter().any(|e| {
                    if e.key == "user.profile.base" {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&e.content) {
                            return parsed
                                .get("display_name")
                                .and_then(|v| v.as_str())
                                .is_some()
                                || (parsed
                                    .get("title")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Visitor")
                                    != "Visitor");
                        }
                    }
                    !e.tags.contains(&"default".to_string())
                });

                if has_real_data {
                    let mut context_parts = Vec::new();
                    for engram in &engrams {
                        context_parts.push(format!("[{}]: {}", engram.key, engram.content));
                    }
                    PersonaLoadResult::Existing(context_parts.join("\n\n"))
                } else {
                    PersonaLoadResult::NewUser
                }
            }
            Ok(_) => PersonaLoadResult::NewUser,
            Err(e) => {
                warn!("Failed to load persona engrams for {}: {}", wallet, e);
                PersonaLoadResult::NewUser
            }
        }
    }

    async fn load_recent_chat_context(&self, wallet: &str) -> Option<String> {
        let client = self.engrams_client.as_ref()?;

        let request = crate::engrams::SearchRequest {
            wallet_address: Some(wallet.to_string()),
            engram_type: Some("conversation".to_string()),
            query: None,
            tags: Some(vec!["session".to_string(), "moros".to_string()]),
            limit: Some(3),
            offset: None,
        };

        match client.search_engrams(request).await {
            Ok(engrams) if !engrams.is_empty() => {
                let mut summaries = Vec::new();
                for engram in &engrams {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&engram.content) {
                        let msg_count = parsed
                            .get("message_count")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        let saved_at = parsed
                            .get("saved_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or(&engram.updated_at);
                        let first_msg = parsed
                            .get("first_user_message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("(unknown)");
                        summaries.push(format!(
                            "- {} ({} messages): \"{}\"",
                            saved_at, msg_count, first_msg
                        ));
                    }
                }
                if summaries.is_empty() {
                    None
                } else {
                    Some(summaries.join("\n"))
                }
            }
            _ => None,
        }
    }

    pub async fn save_session_to_engrams(&self, wallet_address: &str) {
        let client = match self.engrams_client.as_ref() {
            Some(c) => c,
            None => return,
        };

        let history = self.conversation_history.read().await;
        let non_system: Vec<_> = history.iter().filter(|m| m.role != "system").collect();

        if non_system.len() < 2 {
            return;
        }

        let first_user_msg = non_system
            .iter()
            .find(|m| m.role == "user")
            .map(|m| {
                if m.content.len() > 200 {
                    format!("{}...", &m.content[..200])
                } else {
                    m.content.clone()
                }
            })
            .unwrap_or_default();

        let compacted: Vec<serde_json::Value> = non_system
            .iter()
            .map(|m| {
                let truncated = if m.content.len() > 500 {
                    format!("{}...", &m.content[..500])
                } else {
                    m.content.clone()
                };
                json!({ "role": m.role, "content": truncated })
            })
            .collect();

        let session_id = self.current_session_id.as_deref().unwrap_or("unknown");
        let content = json!({
            "session_id": session_id,
            "message_count": non_system.len(),
            "first_user_message": first_user_msg,
            "messages": compacted,
            "saved_at": Utc::now().to_rfc3339()
        });

        let key = format!("chat.session.{}", Uuid::new_v4());

        let request = crate::engrams::CreateEngramRequest {
            wallet_address: wallet_address.to_string(),
            engram_type: "conversation".to_string(),
            key,
            content: content.to_string(),
            metadata: None,
            tags: Some(vec![
                "chat".to_string(),
                "session".to_string(),
                "moros".to_string(),
            ]),
            is_public: Some(false),
        };

        if let Err(e) = client.upsert_engram(request).await {
            warn!("Failed to save Moros session to engrams: {}", e);
        } else {
            info!(
                "💾 Moros chat session saved to engrams for wallet {}",
                &wallet_address[..8.min(wallet_address.len())]
            );
        }
    }

    pub async fn start(&mut self, api_keys: &ApiKeys) -> AppResult<()> {
        info!("🌑 Starting Moros Agent services...");

        info!("🧠 Initializing LLM Service Factory...");
        let mut llm_factory = LLMServiceFactory::new();
        llm_factory.initialize(api_keys).await?;
        let llm_factory_arc = Arc::new(RwLock::new(llm_factory));
        self.llm_factory = Some(llm_factory_arc.clone());
        info!("✅ LLM Service Factory ready");

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

        self.start_new_chat_session();

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

        info!("🔌 Connecting to MCP server...");
        match self.mcp_client.connect().await {
            Ok(_) => {
                info!("✅ MCP client connected successfully");
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

        info!("🎯 Moros Agent ready for convergence operations");

        self.running = true;
        Ok(())
    }

    pub async fn stop(&mut self) -> AppResult<()> {
        info!("🛑 Stopping Moros Agent...");
        self.running = false;
        log_agent_shutdown!("moros");
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

        if lower.contains("mcp")
            && (lower.contains("tool")
                || lower.contains("capability")
                || lower.contains("function"))
        {
            return true;
        }

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
            "tool", "tools", "cleanup", "converge", "resolve", "engram", "memory", "compact",
            "finalize",
        ];

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

        info!("🌑 [Moros] Chat from {}", &user_id[..8.min(user_id.len())]);

        let user_message = ConversationMessage::new(message.clone(), "user".to_string())
            .with_metadata(user_context.clone().unwrap_or_default());

        {
            let mut history = self.conversation_history.write().await;
            history.push(user_message);
        }

        let personality_config = self
            .personalities
            .get(&self.personality)
            .unwrap_or(&self.personalities["unified"]);

        let inject_tools = if Self::is_capability_question(&message) {
            info!("🔧 Capability question detected - injecting MCP tool list");
            Some(self.get_tools_for_prompt().await)
        } else {
            None
        };

        let context = self
            .build_conversation_context(&user_context, inject_tools.as_deref())
            .await;

        let message_clone = message.clone();

        let is_dev = user_context
            .as_ref()
            .and_then(|ctx| ctx.get("is_dev_wallet"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let model_override = if is_dev {
            let premium_model = crate::config::dev_wallet::get_dev_preferred_model();
            info!(
                "🔥 DEV WALLET BOOST - forcing premium model: {}",
                premium_model
            );
            Some(premium_model.to_string())
        } else {
            self.current_model.clone()
        };

        let max_tokens = if is_dev {
            Some(4096)
        } else {
            let free_tier_limit = user_context
                .as_ref()
                .and_then(|ctx| ctx.get("max_output_tokens"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);

            if let Some(limit) = free_tier_limit {
                info!("🆓 Applying free tier output limit: {} tokens", limit);
                Some(limit)
            } else {
                Some(600)
            }
        };

        let tools = user_context
            .as_ref()
            .and_then(|ctx| ctx.get("wallet_address"))
            .and_then(|v| v.as_str())
            .map(|_| Self::build_function_calling_tools());

        let request = LLMRequest {
            prompt: message,
            system_prompt: Some(context.system_prompt),
            messages: Some(context.messages),
            max_tokens,
            temperature: Some(0.7),
            top_p: None,
            stop_sequences: None,
            tools,
            model_override,
            concise: true,
            max_chars: None,
            reasoning: None,
        };

        let requirements = TaskRequirements {
            required_capabilities: vec![ModelCapability::Conversation, ModelCapability::Reasoning],
            optimization_goal: personality_config.optimization_goal.clone(),
            priority: Priority::High,
            task_type: "conversation".to_string(),
            allow_local_models: true,
            preferred_providers: vec!["openrouter".to_string()],
            min_quality_score: Some(0.7),
            max_cost_per_1k_tokens: None,
            min_context_window: None,
        };

        let llm_response = {
            let factory = llm_factory.read().await;
            let result = {
                let user_key_provider = self.resolve_user_key_provider(&user_context);
                if let Some((provider_type, api_key)) = user_key_provider {
                    info!("🔑 Using user's {} API key", provider_type.as_str());
                    factory
                        .generate_with_key(&request, provider_type, &api_key)
                        .await
                } else {
                    factory.generate(&request, Some(requirements.clone())).await
                }
            };
            drop(factory);

            match result {
                Ok(response) => response,
                Err(e) => {
                    let error_msg = e.to_string().to_lowercase();

                    if error_msg.contains("maximum context length")
                        || error_msg.contains("context length")
                        || error_msg.contains("too many tokens")
                        || error_msg.contains("reduce the length")
                        || error_msg.contains("middle-out")
                    {
                        warn!("⚠️ Context limit exceeded, auto-compacting conversation and retrying...");

                        self.trim_conversation_history().await;

                        let context = self
                            .build_conversation_context(&user_context, inject_tools.as_deref())
                            .await;

                        let retry_request = LLMRequest {
                            prompt: message_clone.clone(),
                            system_prompt: Some(context.system_prompt),
                            messages: Some(context.messages),
                            max_tokens,
                            temperature: Some(0.7),
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
                        factory
                            .generate(&retry_request, Some(requirements.clone()))
                            .await?
                    } else {
                        return Err(e);
                    }
                }
            }
        };

        let llm_response = if let Some(ref tool_calls) = llm_response.tool_calls {
            if !tool_calls.is_empty() {
                let wallet_address = user_context
                    .as_ref()
                    .and_then(|ctx| ctx.get("wallet_address"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("anonymous");

                let mut tool_results = Vec::new();
                for tc in tool_calls {
                    let name = tc
                        .get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown");
                    let args_str = tc
                        .get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(|a| a.as_str())
                        .unwrap_or("{}");

                    let mut args: serde_json::Value =
                        serde_json::from_str(args_str).unwrap_or(json!({}));
                    if let Some(obj) = args.as_object_mut() {
                        obj.insert("wallet_address".to_string(), json!(wallet_address));
                    }

                    info!("🔧 Executing function call: {} with args: {}", name, args);

                    if name == "moros_set_model" {
                        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                        let result = self.handle_set_model_tool(query).await;
                        tool_results.push(format!("{}: {}", name, result));
                    } else if let Some(engrams_client) = &self.engrams_client {
                        let result =
                            crate::mcp::handlers::execute_tool_with_engrams(engrams_client, name, args).await;
                        let result_text = result
                            .content
                            .first()
                            .map(|c| c.text.clone())
                            .unwrap_or_else(|| "Tool executed".to_string());
                        tool_results.push(format!("{}: {}", name, result_text));
                    }
                }

                if !tool_results.is_empty() {
                    let original_text = llm_response.content.clone();
                    let tool_context = format!(
                        "{}\n\nTOOL RESULTS (saved, acknowledge naturally):\n{}",
                        original_text,
                        tool_results.join("\n")
                    );

                    let followup_request = LLMRequest {
                        prompt: message_clone.clone(),
                        system_prompt: Some(tool_context),
                        messages: None,
                        max_tokens,
                        temperature: Some(0.7),
                        top_p: None,
                        stop_sequences: None,
                        tools: None,
                        model_override: self.current_model.clone(),
                        concise: true,
                        max_chars: None,
                        reasoning: None,
                    };

                    let factory = llm_factory.read().await;
                    match factory
                        .generate(&followup_request, Some(requirements.clone()))
                        .await
                    {
                        Ok(followup) => followup,
                        Err(e) => {
                            warn!("⚠️ Tool followup failed: {}, using original response", e);
                            llm_response
                        }
                    }
                } else {
                    llm_response
                }
            } else {
                llm_response
            }
        } else {
            llm_response
        };

        let cleaned_content = self.strip_thinking_tags(&llm_response.content);
        let latency_ms = start_time.elapsed().as_millis() as f64;

        self.current_model = Some(llm_response.model_used.clone());

        let assistant_message =
            ConversationMessage::new(cleaned_content.clone(), "assistant".to_string())
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
                    meta
                });

        {
            let mut history = self.conversation_history.write().await;
            history.push(assistant_message);
        }

        self.trim_conversation_history().await;

        let confidence_score = self.calculate_confidence(&llm_response);

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

    pub async fn get_model_status(&self) -> AppResult<serde_json::Value> {
        if let Some(llm_factory) = &self.llm_factory {
            let factory = llm_factory.read().await;
            let health = factory.health_check().await?;
            let stats = factory.get_stats().await;

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
                Ok(models) => models.iter().any(|model| {
                    model
                        .get("id")
                        .and_then(|id| id.as_str())
                        .map(|id| id == model_name)
                        .unwrap_or(false)
                }),
                Err(_) => model_name.contains("/") || model_name.contains(":"),
            }
        } else {
            false
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

        info!("💬 Moros conversation history cleared");

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

    fn start_new_chat_session(&mut self) {
        self.current_session_id = Some(format!(
            "moros_session_{}",
            Utc::now().format("%Y%m%d_%H%M%S")
        ));
        info!(
            "💬 Started new Moros chat session: {:?}",
            self.current_session_id
        );
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

        if let Some(tools_list) = inject_tools {
            base_system_prompt.push_str(&format!("\n\nAVAILABLE MCP TOOLS:\n{}\n\nWhen asked about capabilities, reference these specific tools.", tools_list));
        }

        base_system_prompt.push_str("\n\nMEMORY PROTOCOL:\n- Proactively remember important things visitors tell you (use moros_remember)\n- When visitors share preferences, facts, or decisions, save them using tools without asking permission\n- Use user_profile_update to save profile details like display_name, bio, experience level");

        if let Some(context) = user_context {
            if let Some(wallet_address) = context.get("wallet_address").and_then(|v| v.as_str()) {
                match self.load_persona_context(wallet_address).await {
                    PersonaLoadResult::Existing(persona_context) => {
                        base_system_prompt.push_str(&format!(
                            "\n\nUSER PERSONA CONTEXT (from engram memory):\n{}",
                            persona_context
                        ));
                    }
                    PersonaLoadResult::NewUser => {
                        base_system_prompt.push_str(&format!("\n\n{}", NEW_USER_SYSTEM_PROMPT));
                    }
                }

                if let Some(recent_context) = self.load_recent_chat_context(wallet_address).await {
                    base_system_prompt.push_str(&format!(
                        "\n\nRECENT CONVERSATION HISTORY (from previous sessions):\n{}",
                        recent_context
                    ));
                }
            }
        }

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

    fn build_function_calling_tools() -> Vec<serde_json::Value> {
        vec![
            json!({
                "type": "function",
                "function": {
                    "name": "user_profile_update",
                    "description": "Save or update a user profile field. Use this when you learn something about the visitor.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "field": {
                                "type": "string",
                                "description": "Profile field name: 'base' for display_name/bio/title, 'interests' for interests, 'experience' for experience level"
                            },
                            "content": {
                                "type": "string",
                                "description": "JSON string content for the profile field"
                            }
                        },
                        "required": ["field", "content"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "moros_remember",
                    "description": "Proactively save important context about the visitor. Use when they share preferences, facts, decisions, or anything worth remembering.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "key": {
                                "type": "string",
                                "description": "Dot-path key for the memory (e.g., 'visitor.preference.chains', 'visitor.fact.role')"
                            },
                            "content": {
                                "type": "string",
                                "description": "The information to remember"
                            },
                            "engram_type": {
                                "type": "string",
                                "description": "Type of memory: persona, preference, or knowledge",
                                "enum": ["persona", "preference", "knowledge"]
                            }
                        },
                        "required": ["key", "content"]
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "moros_cleanup",
                    "description": "Compact old conversation sessions. Keeps 5 most recent and all pinned sessions.",
                    "parameters": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    }
                }
            }),
            json!({
                "type": "function",
                "function": {
                    "name": "moros_set_model",
                    "description": "Switch the AI model when the user requests a different model. Search for models by name (e.g., 'opus', 'claude', 'gpt-4', 'deepseek'). Returns the best match and alternatives if no exact match.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Model name or keyword to search for (e.g., 'opus', 'claude-sonnet', 'gpt-4o', 'deepseek')"
                            }
                        },
                        "required": ["query"]
                    }
                }
            }),
        ]
    }

    async fn handle_set_model_tool(&mut self, query: &str) -> String {
        let query_lower = query.to_lowercase();

        let factory = match &self.llm_factory {
            Some(f) => f,
            None => return "LLM factory not initialized".to_string(),
        };

        let models = {
            let factory = factory.read().await;
            factory.fetch_available_models().await.unwrap_or_default()
        };

        let mut matches: Vec<(String, String, f64)> = Vec::new();
        for model in &models {
            let id = model.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let name = model.get("name").and_then(|v| v.as_str()).unwrap_or(id);
            let id_lower = id.to_lowercase();
            let name_lower = name.to_lowercase();

            if id_lower == query_lower || name_lower == query_lower {
                matches.push((id.to_string(), name.to_string(), 1.0));
            } else if id_lower.contains(&query_lower) || name_lower.contains(&query_lower) {
                let score = if id_lower.starts_with(&query_lower) {
                    0.9
                } else {
                    0.7
                };
                matches.push((id.to_string(), name.to_string(), score));
            }
        }

        matches.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        if matches.is_empty() {
            return format!("No models found matching '{}'. Try a different search term like 'claude', 'gpt', 'deepseek', 'llama', or 'gemini'.", query);
        }

        let best = &matches[0];
        self.current_model = Some(best.0.clone());
        self.preferred_model = best.0.clone();

        let mut result = format!("Switched to model: {} ({})", best.1, best.0);
        if matches.len() > 1 {
            let alternatives: Vec<String> = matches[1..matches.len().min(4)]
                .iter()
                .map(|m| format!("{} ({})", m.1, m.0))
                .collect();
            result.push_str(&format!("\nOther matches: {}", alternatives.join(", ")));
        }

        info!("🔄 Model switched via function call to: {}", best.0);
        result
    }

    fn resolve_user_key_provider(
        &self,
        user_context: &Option<HashMap<String, serde_json::Value>>,
    ) -> Option<(crate::models::ModelProvider, String)> {
        let ctx = user_context.as_ref()?;
        let model = self.current_model.as_deref()?;

        let provider_type = if model.contains("claude") || model.contains("anthropic/") {
            crate::models::ModelProvider::Anthropic
        } else if model.starts_with("openai/") || model.contains("gpt") {
            crate::models::ModelProvider::OpenAI
        } else if model.contains("groq/") {
            crate::models::ModelProvider::Groq
        } else {
            crate::models::ModelProvider::OpenRouter
        };

        let key_field = format!("user_api_key_{}", provider_type.as_str());
        let api_key = ctx.get(&key_field)?.as_str()?.to_string();

        Some((provider_type, api_key))
    }

    async fn trim_conversation_history(&mut self) {
        let mut history = self.conversation_history.write().await;

        let total_tokens: usize = history.iter().map(|msg| (msg.content.len() / 4) + 10).sum();

        if total_tokens > self.context_limit {
            let mut system_messages: Vec<ConversationMessage> = Vec::new();
            let mut conversation_messages: Vec<ConversationMessage> = Vec::new();

            for msg in history.iter() {
                if msg.role == "system" {
                    system_messages.push(msg.clone());
                } else {
                    conversation_messages.push(msg.clone());
                }
            }

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
                "✂️ Trimmed Moros conversation history to {} messages",
                history.len()
            );
        }
    }

    fn strip_thinking_tags(&self, content: &str) -> String {
        let re = regex::Regex::new(r"(?s)<think>.*?</think>").unwrap();
        let mut cleaned = re.replace_all(content, "").to_string();

        let whitespace_re = regex::Regex::new(r"\n\s*\n\s*\n").unwrap();
        cleaned = whitespace_re.replace_all(&cleaned, "\n\n").to_string();

        cleaned.trim().to_string()
    }

    fn calculate_confidence(&self, llm_response: &crate::models::LLMResponse) -> f64 {
        let mut confidence: f64 = 0.8;

        match llm_response.finish_reason.as_str() {
            "stop" => confidence += 0.1,
            "length" => confidence -= 0.1,
            _ => {}
        }

        let content_length = llm_response.content.len();
        if (50..=1000).contains(&content_length) {
            confidence += 0.05;
        } else if content_length < 10 {
            confidence -= 0.2;
        }

        confidence.clamp(0.0, 1.0)
    }

    pub async fn register_agent(&mut self, agent_repo: &AgentRepository) -> AppResult<()> {
        let capabilities = vec![
            "conversation".to_string(),
            "finalization".to_string(),
            "resolution".to_string(),
            "reasoning".to_string(),
            "cleanup".to_string(),
        ];

        match agent_repo
            .get_by_name_and_type("moros", "conversational")
            .await
        {
            Ok(Some(existing_agent)) => {
                info!(
                    "✅ MOROS convergence executor registered with ID: {}",
                    existing_agent.id
                );
                self.agent_id = Some(existing_agent.id);

                if let Err(e) = agent_repo
                    .update_health_status(&existing_agent.id, "healthy")
                    .await
                {
                    warn!("⚠️ Failed to update MOROS health status: {}", e);
                }
            }
            Ok(None) => {
                info!("📝 Registering MOROS convergence executor in database...");
                match agent_repo
                    .create(
                        "moros",
                        "conversational",
                        Some("MOROS - Executor of Final Convergence for the NullBlock mesh"),
                        &capabilities,
                    )
                    .await
                {
                    Ok(agent_entity) => {
                        info!(
                            "✅ MOROS convergence executor registered with ID: {}",
                            agent_entity.id
                        );
                        self.agent_id = Some(agent_entity.id);
                    }
                    Err(e) => {
                        error!("❌ Failed to register MOROS convergence executor: {}", e);
                        return Err(AppError::DatabaseError(format!(
                            "Agent registration failed: {}",
                            e
                        )));
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to check existing MOROS registration: {}", e);
                return Err(AppError::DatabaseError(format!(
                    "Agent lookup failed: {}",
                    e
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct ConversationContext {
    system_prompt: String,
    messages: Vec<HashMap<String, String>>,
}
