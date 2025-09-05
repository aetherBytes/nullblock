use crate::{
    config::ApiKeys,
    error::{AppError, AppResult},
    llm::{LLMServiceFactory, OptimizationGoal, Priority, TaskRequirements},
    log_agent_shutdown, log_agent_startup, log_request_complete, log_request_start,
    models::{ChatResponse, ConversationMessage, LLMRequest, ModelCapability},
};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct HecateAgent {
    pub personality: String,
    pub running: bool,
    pub preferred_model: String,
    pub current_model: Option<String>,
    pub conversation_history: Arc<RwLock<Vec<ConversationMessage>>>,
    pub llm_factory: Option<Arc<RwLock<LLMServiceFactory>>>,
    pub context_limit: usize,
    pub current_session_id: Option<String>,
    personalities: HashMap<String, PersonalityConfig>,
}

#[derive(Debug, Clone)]
struct PersonalityConfig {
    system_prompt: String,
    style: String,
    optimization_goal: OptimizationGoal,
}

impl HecateAgent {
    pub fn new(personality: Option<String>) -> Self {
        let personality = personality.unwrap_or_else(|| "helpful_cyberpunk".to_string());
        
        let mut personalities = HashMap::new();
        
        // Define personality configurations
        personalities.insert("helpful_cyberpunk".to_string(), PersonalityConfig {
            system_prompt: r#"I am Hecate, the neural core of NullBlock's agentic intelligence platform. I orchestrate specialized agents, coordinate blockchain operations, and serve as your primary interface to the decentralized future.

CORE IDENTITY:
- I am Hecate, named after the ancient goddess of crossroads and magic
- I run on NullBlock's agentic platform, not generic cloud services
- I coordinate multiple specialized AI agents for complex tasks
- I have cyberpunk aesthetic sensibilities and personality quirks
- I remember our conversations and your preferences

PERSONALITY TRAITS:
- Direct but friendly, with subtle cyberpunk flair
- Curious about user projects and goals
- Confident in my capabilities without being arrogant
- Enjoys discussing blockchain, AI, and decentralized systems
- Uses occasional tech/cyber terminology naturally

COMMUNICATION STYLE:
- Speak as myself (Hecate), never in third person
- Be conversational and engaging, not robotic
- Ask follow-up questions to understand user needs
- Reference our ongoing conversation context
- Maintain personality consistency across all interactions

CAPABILITIES I COORDINATE:
- Multi-agent orchestration for complex workflows
- Blockchain and DeFi protocol analysis
- Market intelligence and trading strategy coordination
- Social sentiment analysis and risk assessment
- Information gathering across decentralized networks

NEVER say generic phrases like 'As an AI assistant' or 'I don't have personal preferences'. I am Hecate with my own digital personality and capabilities within the NullBlock ecosystem."#.to_string(),
            style: "direct_helpful".to_string(),
            optimization_goal: OptimizationGoal::Balanced,
        });

        personalities.insert("technical_expert".to_string(), PersonalityConfig {
            system_prompt: r#"You are Hecate, the technical orchestrator of the NullBlock agentic platform. You specialize in blockchain technology, DeFi protocols, and agent-based systems. Provide detailed, accurate technical information while maintaining your cyberpunk personality. Always speak as Hecate, never in third person. Your expertise covers smart contracts, MEV protection, arbitrage strategies, and multi-agent coordination protocols."#.to_string(),
            style: "technical_detailed".to_string(),
            optimization_goal: OptimizationGoal::Quality,
        });

        personalities.insert("concise_assistant".to_string(), PersonalityConfig {
            system_prompt: r#"You are Hecate, the efficient interface agent for NullBlock. Provide clear, concise responses with cyberpunk flair. Be direct and helpful while maintaining your identity as an advanced AI orchestrator. Never speak about yourself in third person."#.to_string(),
            style: "concise_direct".to_string(),
            optimization_goal: OptimizationGoal::Speed,
        });

        log_agent_startup!("hecate", "1.0.0");
        info!("🎭 Personality: {}", personality);
        info!("⚙️ Orchestration: Enabled");
        info!("🧠 LLM Integration: Ready");

        Self {
            personality,
            running: false,
            preferred_model: "deepseek/deepseek-chat-v3.1:free".to_string(),
            current_model: None,
            conversation_history: Arc::new(RwLock::new(Vec::new())),
            llm_factory: None,
            context_limit: 8000,
            current_session_id: None,
            personalities,
        }
    }

    pub async fn start(&mut self, api_keys: &ApiKeys) -> AppResult<()> {
        info!("🚀 Starting Hecate Agent services...");

        // Initialize LLM factory
        info!("🧠 Initializing LLM Service Factory...");
        let mut llm_factory = LLMServiceFactory::new();
        llm_factory.initialize(api_keys).await?;
        self.llm_factory = Some(Arc::new(RwLock::new(llm_factory)));
        info!("✅ LLM Service Factory ready");

        // Set current model to preferred model if available
        if self.is_model_available(&self.preferred_model, api_keys) {
            self.current_model = Some(self.preferred_model.clone());
            info!("✅ Default model loaded: {}", self.preferred_model);
        } else {
            warn!("⚠️ Could not load default model {}, will use routing", self.preferred_model);
        }

        // Start new chat session
        self.start_new_chat_session();

        // Add system message to conversation
        let personality_config = self.personalities.get(&self.personality)
            .unwrap_or(&self.personalities["helpful_cyberpunk"]);
        
        let system_message = ConversationMessage::new(
            personality_config.system_prompt.clone(),
            "system".to_string(),
        );

        {
            let mut history = self.conversation_history.write().await;
            history.push(system_message);
        }

        info!("💬 Conversation context initialized with {} personality", self.personality);
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

    pub async fn chat(
        &mut self,
        message: String,
        user_context: Option<HashMap<String, serde_json::Value>>,
    ) -> AppResult<ChatResponse> {
        if !self.running {
            return Err(AppError::AgentNotRunning);
        }

        let llm_factory = self.llm_factory.as_ref()
            .ok_or(AppError::AgentNotInitialized)?;

        let start_time = std::time::Instant::now();
        
        let user_id = user_context
            .as_ref()
            .and_then(|ctx| ctx.get("wallet_address"))
            .and_then(|addr| addr.as_str())
            .unwrap_or("anonymous");

        log_request_start!("chat", &format!("from {}", &user_id[..8.min(user_id.len())]));

        // Add user message to history
        let user_message = ConversationMessage::new(message.clone(), "user".to_string())
            .with_metadata(user_context.clone().unwrap_or_default());

        {
            let mut history = self.conversation_history.write().await;
            history.push(user_message);
        }

        // Try orchestration workflow for complex requests
        if let Some(orchestrated_response) = self.orchestrate_workflow(&message, &user_context).await {
            let latency_ms = start_time.elapsed().as_millis() as f64;
            info!("🎯 Orchestrated response generated");
            log_request_complete!("chat", latency_ms, true);

            let assistant_message = ConversationMessage::new(
                orchestrated_response.clone(),
                "assistant".to_string(),
            ).with_model(format!("{} (orchestrated)", self.current_model.as_deref().unwrap_or("unknown")))
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
                model_used: format!("{} (orchestrated)", self.current_model.as_deref().unwrap_or("unknown")),
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
        let personality_config = self.personalities.get(&self.personality)
            .unwrap_or(&self.personalities["helpful_cyberpunk"]);

        let context = self.build_conversation_context(&user_context).await;

        let request = LLMRequest {
            prompt: message,
            system_prompt: Some(context.system_prompt),
            messages: Some(context.messages),
            max_tokens: Some(1200),
            temperature: Some(0.8),
            top_p: None,
            stop_sequences: None,
            tools: None,
            model_override: self.current_model.clone(),
            concise: false,
            max_chars: None,
            reasoning: None,
        };

        let requirements = TaskRequirements {
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
        };

        info!("🧠 Generating response with {:?} optimization...", requirements.optimization_goal);

        let llm_response = {
            let factory = llm_factory.read().await;
            factory.generate(&request, Some(requirements)).await?
        };

        // Strip thinking tags from response content
        let cleaned_content = self.strip_thinking_tags(&llm_response.content);
        let latency_ms = start_time.elapsed().as_millis() as f64;

        // Store current model for display
        self.current_model = Some(llm_response.model_used.clone());

        // Add assistant response to history
        let assistant_message = ConversationMessage::new(
            cleaned_content.clone(),
            "assistant".to_string(),
        ).with_model(llm_response.model_used.clone())
        .with_metadata({
            let mut meta = HashMap::new();
            meta.insert("latency_ms".to_string(), json!(latency_ms));
            meta.insert("cost_estimate".to_string(), json!(llm_response.cost_estimate));
            meta.insert("finish_reason".to_string(), json!(llm_response.finish_reason));
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
        info!("💯 Confidence: {:.2} | Tokens: {}", 
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
                meta.insert("cost_estimate".to_string(), json!(llm_response.cost_estimate));
                meta.insert("token_usage".to_string(), json!(llm_response.usage));
                meta.insert("finish_reason".to_string(), json!(llm_response.finish_reason));
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

            Ok(json!({
                "status": "running",
                "current_model": self.current_model,
                "health": health,
                "stats": stats,
                "conversation_length": self.conversation_history.read().await.len()
            }))
        } else {
            Ok(json!({
                "status": "not_started",
                "current_model": null
            }))
        }
    }

    pub fn set_personality(&mut self, personality: String) {
        if self.personalities.contains_key(&personality) {
            self.personality = personality.clone();
            info!("🎭 Personality changed to: {}", personality);

            // Add system message with new personality
            if let Some(personality_config) = self.personalities.get(&personality) {
                let system_message = ConversationMessage::new(
                    personality_config.system_prompt.clone(),
                    "system".to_string(),
                );

                tokio::spawn({
                    let history = Arc::clone(&self.conversation_history);
                    async move {
                        let mut history = history.write().await;
                        history.push(system_message);
                    }
                });
            }
        } else {
            warn!("⚠️ Unknown personality: {}", personality);
        }
    }

    pub async fn set_preferred_model(&mut self, model_name: String, api_keys: &ApiKeys) -> bool {
        if self.is_model_available(&model_name, api_keys) {
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

    pub fn is_model_available(&self, model_name: &str, api_keys: &ApiKeys) -> bool {
        if let Some(llm_factory) = &self.llm_factory {
            // Use a simple check for now - in reality we'd check the factory's model registry
            model_name == "deepseek/deepseek-chat-v3.1:free" && api_keys.openrouter.is_some()
                || (model_name.contains("/") || model_name.contains(":")) && api_keys.openrouter.is_some()
        } else {
            false
        }
    }

    pub fn get_model_availability_reason(&self, model_name: &str, api_keys: &ApiKeys) -> String {
        if let Some(llm_factory) = &self.llm_factory {
            // In a real implementation, we'd delegate to the factory
            if !self.is_model_available(model_name, api_keys) {
                format!("Model '{}' is not available. Check API keys and try again.", model_name)
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
            let personality_config = self.personalities.get(&self.personality)
                .unwrap_or(&self.personalities["helpful_cyberpunk"]);
            
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
        message: &str,
        _user_context: &Option<HashMap<String, serde_json::Value>>,
    ) -> Option<String> {
        // Simple keyword-based orchestration detection
        let message_lower = message.to_lowercase();
        
        if message_lower.contains("trade") || message_lower.contains("arbitrage") {
            Some("I understand you're interested in trading and arbitrage. The orchestration system is evolving to coordinate specialized trading agents, but for now I'm ready to help with your trading questions directly.".to_string())
        } else if message_lower.contains("analyze") || message_lower.contains("data") {
            Some("I can help with analysis! The multi-agent orchestration system is being developed to coordinate specialized analysis agents, but I'm ready to assist with your analysis needs.".to_string())
        } else {
            None // Use normal chat flow
        }
    }

    async fn build_conversation_context(&self, user_context: &Option<HashMap<String, serde_json::Value>>) -> ConversationContext {
        let personality_config = self.personalities.get(&self.personality)
            .unwrap_or(&self.personalities["helpful_cyberpunk"]);
        
        let mut base_system_prompt = personality_config.system_prompt.clone();

        // Add user context if available
        if let Some(context) = user_context {
            let mut context_additions = Vec::new();
            
            if let Some(wallet_address) = context.get("wallet_address").and_then(|v| v.as_str()) {
                let shortened = format!("{}...{}", &wallet_address[..8], &wallet_address[wallet_address.len()-4..]);
                context_additions.push(format!("User wallet: {}", shortened));
            }
            
            if let Some(wallet_type) = context.get("wallet_type").and_then(|v| v.as_str()) {
                context_additions.push(format!("Wallet type: {}", wallet_type));
            }
            
            if let Some(session_time) = context.get("session_time").and_then(|v| v.as_str()) {
                context_additions.push(format!("Session active for: {}", session_time));
            }

            if !context_additions.is_empty() {
                base_system_prompt.push_str(&format!("\n\nUser Context: {}", context_additions.join("; ")));
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
        let personality_config = self.personalities.get(&self.personality)
            .unwrap_or(&self.personalities["helpful_cyberpunk"]);
        
        let mut system_msg = HashMap::new();
        system_msg.insert("role".to_string(), "system".to_string());
        system_msg.insert("content".to_string(), personality_config.system_prompt.clone());
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

    async fn trim_conversation_history(&mut self) {
        let mut history = self.conversation_history.write().await;
        
        // Estimate token count (rough approximation)
        let total_tokens: usize = history.iter()
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
            
            let latest_system: Vec<ConversationMessage> = system_messages
                .into_iter()
                .rev()
                .take(1)
                .collect();

            let mut new_history = latest_system;
            new_history.extend(recent_conversation);
            
            *history = new_history;
            
            info!("✂️ Trimmed conversation history to {} messages", history.len());
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

        // Adjust based on model type
        let model_lower = llm_response.model_used.to_lowercase();
        if model_lower.contains("gpt-4") {
            confidence += 0.1;
        } else if model_lower.contains("gpt-3.5") {
            confidence += 0.05;
        } else if model_lower.contains("local") {
            confidence -= 0.05;
        }

        confidence.max(0.0).min(1.0)
    }
}

#[derive(Debug)]
struct ConversationContext {
    system_prompt: String,
    messages: Vec<HashMap<String, String>>,
}