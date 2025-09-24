use crate::{
    config::ApiKeys,
    database::repositories::AgentRepository,
    error::{AppError, AppResult},
    llm::{LLMServiceFactory, OptimizationGoal, Priority, TaskRequirements},
    log_agent_shutdown, log_agent_startup, log_request_complete, log_request_start,
    models::{ConversationMessage, LLMRequest, ModelCapability},
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

pub struct MarketingAgent {
    pub personality: String,
    pub running: bool,
    pub preferred_model: String,
    pub current_model: Option<String>,
    pub conversation_history: Arc<RwLock<Vec<ConversationMessage>>>,
    pub llm_factory: Option<Arc<RwLock<LLMServiceFactory>>>,
    pub context_limit: usize,
    pub current_session_id: Option<String>,
    pub agent_id: Option<uuid::Uuid>,
    pub twitter_api_key: Option<String>,
    pub twitter_api_secret: Option<String>,
    pub twitter_access_token: Option<String>,
    pub twitter_access_secret: Option<String>,
    pub content_themes: HashMap<String, ContentTheme>,
    pub posting_schedule: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ContentTheme {
    pub name: String,
    pub description: String,
    pub hashtags: Vec<String>,
    pub tone: String,
    pub target_audience: String,
    pub content_templates: Vec<String>,
}

impl MarketingAgent {
    pub fn new(personality: Option<String>) -> Self {
        let personality = personality.unwrap_or_else(|| "cyberpunk_marketer".to_string());
        
        let mut content_themes = HashMap::new();
        
        // Define content themes for different types of marketing content
        content_themes.insert("product_announcement".to_string(), ContentTheme {
            name: "Product Announcement".to_string(),
            description: "Announcing new features, releases, or capabilities".to_string(),
            hashtags: vec!["#NullBlock".to_string(), "#AgenticAI".to_string(), "#DeFi".to_string(), "#Web3".to_string()],
            tone: "excited_technical".to_string(),
            target_audience: "developers_builders".to_string(),
            content_templates: vec![
                "🚀 Just shipped {feature} to {platform}! {description} {hashtags}".to_string(),
                "⚡ {platform} update: {feature} is now live! {description} {hashtags}".to_string(),
                "🔧 Building the future of {domain}: {feature} is here! {description} {hashtags}".to_string(),
            ],
        });

        content_themes.insert("technical_insight".to_string(), ContentTheme {
            name: "Technical Insight".to_string(),
            description: "Sharing technical knowledge and insights".to_string(),
            hashtags: vec!["#Rust".to_string(), "#AI".to_string(), "#Blockchain".to_string(), "#Tech".to_string()],
            tone: "educational_authoritative".to_string(),
            target_audience: "technical_community".to_string(),
            content_templates: vec![
                "🧠 Deep dive: {topic} in {context}. {insight} {hashtags}".to_string(),
                "⚙️ Technical insight: {topic} {explanation} {hashtags}".to_string(),
                "🔬 Exploring {topic}: {insight} {hashtags}".to_string(),
            ],
        });

        content_themes.insert("community_engagement".to_string(), ContentTheme {
            name: "Community Engagement".to_string(),
            description: "Engaging with the community and ecosystem".to_string(),
            hashtags: vec!["#Community".to_string(), "#Builders".to_string(), "#OpenSource".to_string(), "#Web3".to_string()],
            tone: "friendly_engaging".to_string(),
            target_audience: "community_members".to_string(),
            content_templates: vec![
                "💬 {question} What's your take on {topic}? {hashtags}".to_string(),
                "🤝 Shoutout to {community} for {reason}! {hashtags}".to_string(),
                "🎯 Building together: {message} {hashtags}".to_string(),
            ],
        });

        content_themes.insert("milestone_celebration".to_string(), ContentTheme {
            name: "Milestone Celebration".to_string(),
            description: "Celebrating achievements and milestones".to_string(),
            hashtags: vec!["#Milestone".to_string(), "#Achievement".to_string(), "#Progress".to_string(), "#NullBlock".to_string()],
            tone: "celebratory_proud".to_string(),
            target_audience: "general_audience".to_string(),
            content_templates: vec![
                "🎉 {milestone} achieved! {description} {hashtags}".to_string(),
                "🏆 {achievement}! {description} {hashtags}".to_string(),
                "✨ {milestone} milestone reached! {description} {hashtags}".to_string(),
            ],
        });

        content_themes.insert("thought_leadership".to_string(), ContentTheme {
            name: "Thought Leadership".to_string(),
            description: "Sharing insights on industry trends and future vision".to_string(),
            hashtags: vec!["#FutureOfAI".to_string(), "#Web3".to_string(), "#Innovation".to_string(), "#ThoughtLeadership".to_string()],
            tone: "visionary_insightful".to_string(),
            target_audience: "industry_leaders".to_string(),
            content_templates: vec![
                "🔮 The future of {domain}: {insight} {hashtags}".to_string(),
                "💡 {perspective} on {topic}: {insight} {hashtags}".to_string(),
                "🚀 {vision} for {domain}: {insight} {hashtags}".to_string(),
            ],
        });

        // Initialize posting schedule
        let mut posting_schedule = HashMap::new();
        posting_schedule.insert("morning".to_string(), "09:00".to_string());
        posting_schedule.insert("afternoon".to_string(), "15:00".to_string());
        posting_schedule.insert("evening".to_string(), "19:00".to_string());

        log_agent_startup!("marketing", "1.0.0");
        info!("🎭 Personality: {}", personality);
        info!("📱 Twitter Integration: Ready");
        info!("📝 Content Themes: {} configured", content_themes.len());

        Self {
            personality,
            running: false,
            preferred_model: "x-ai/grok-4-fast:free".to_string(),
            current_model: None,
            conversation_history: Arc::new(RwLock::new(Vec::new())),
            llm_factory: None,
            context_limit: 8000,
            current_session_id: None,
            agent_id: None,
            twitter_api_key: None,
            twitter_api_secret: None,
            twitter_access_token: None,
            twitter_access_secret: None,
            content_themes,
            posting_schedule,
        }
    }

    pub async fn start(&mut self, api_keys: &ApiKeys) -> AppResult<()> {
        info!("🚀 Starting Marketing Agent services...");

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
        let system_prompt = self.build_system_prompt();
        let system_message = ConversationMessage::new(system_prompt, "system".to_string());

        {
            let mut history = self.conversation_history.write().await;
            history.push(system_message);
        }

        info!("💬 Conversation context initialized with marketing personality");
        info!("🎯 Marketing Agent ready for content creation and social media management");

        self.running = true;
        Ok(())
    }

    pub async fn stop(&mut self) -> AppResult<()> {
        info!("🛑 Stopping Marketing Agent...");
        self.running = false;
        log_agent_shutdown!("marketing");
        Ok(())
    }

    pub async fn generate_content(
        &mut self,
        content_type: String,
        context: Option<HashMap<String, serde_json::Value>>,
    ) -> AppResult<MarketingContent> {
        if !self.running {
            return Err(AppError::AgentNotRunning);
        }

        let llm_factory = self.llm_factory.as_ref()
            .ok_or(AppError::AgentNotInitialized)?;

        let start_time = std::time::Instant::now();
        
        log_request_start!("content_generation", &content_type);

        // Build content generation prompt based on type and context
        let prompt = self.build_content_prompt(&content_type, &context).await;
        
        let request = LLMRequest {
            prompt,
            system_prompt: Some(self.build_system_prompt()),
            messages: Some(self.build_messages_history().await),
            max_tokens: Some(500),
            temperature: Some(0.8),
            top_p: None,
            stop_sequences: None,
            tools: None,
            model_override: self.current_model.clone(),
            concise: false,
            max_chars: Some(280), // Twitter character limit
            reasoning: None,
        };

        let requirements = TaskRequirements {
            required_capabilities: vec![
                ModelCapability::Creative,
                ModelCapability::Conversation,
            ],
            optimization_goal: OptimizationGoal::Quality,
            priority: Priority::High,
            task_type: "content_generation".to_string(),
            allow_local_models: true,
            preferred_providers: vec!["openrouter".to_string()],
            min_quality_score: Some(0.8),
            max_cost_per_1k_tokens: None,
            min_context_window: None,
        };

        info!("🧠 Generating {} content...", content_type);

        let llm_response = {
            let factory = llm_factory.read().await;
            factory.generate(&request, Some(requirements)).await?
        };

        let latency_ms = start_time.elapsed().as_millis() as f64;
        let content = self.parse_generated_content(&llm_response.content, &content_type).await;

        log_request_complete!("content_generation", latency_ms, true);
        info!("✅ Content generated successfully");

        Ok(content)
    }

    pub async fn create_twitter_post(
        &mut self,
        content: String,
        _media_urls: Option<Vec<String>>,
    ) -> AppResult<TwitterPostResult> {
        if !self.running {
            return Err(AppError::AgentNotRunning);
        }

        // For now, simulate Twitter posting
        // In a real implementation, this would use the Twitter API
        info!("📱 Creating Twitter post: {}", content);
        
        // Simulate API call delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        Ok(TwitterPostResult {
            success: true,
            post_id: Some(format!("post_{}", Uuid::new_v4())),
            url: Some(format!("https://twitter.com/nullblock_io/status/{}", Uuid::new_v4())),
            error: None,
        })
    }

    pub async fn analyze_project_progress(&mut self) -> AppResult<ProjectAnalysis> {
        if !self.running {
            return Err(AppError::AgentNotRunning);
        }

        // Analyze current project state for marketing opportunities
        let analysis_prompt = r#"
        Analyze the current state of the NullBlock project and identify key marketing opportunities:

        Current Project Status:
        - Multi-agent orchestration platform with persistent task management
        - Erebus unified router (Port 3000) - GOLDEN RULE architecture
        - Hecate conversational agent with LLM integration
        - Crossroads marketplace system for AI services
        - Task management system with PostgreSQL and Kafka
        - Production-ready components: MCP server, agents, Erebus, Crossroads, Hecate frontend

        Key Features:
        - Agent orchestration and coordination
        - Unified routing through Erebus
        - Marketplace for AI services
        - Protocol agnostic (MCP, A2A, custom)
        - Real-time WebSocket communication
        - Task lifecycle management

        Technology Stack:
        - Rust (performance and reliability)
        - TypeScript/React (frontend)
        - Python (MCP server)
        - PostgreSQL (persistence)
        - Kafka (event streaming)

        Current Development Focus:
        1. Task & Scheduling Infrastructure
        2. Agent Service Integration  
        3. X Marketing Agent (this agent!)

        Identify 3-5 key marketing opportunities and content ideas based on this analysis.
        "#;

        let request = LLMRequest {
            prompt: analysis_prompt.to_string(),
            system_prompt: Some(self.build_system_prompt()),
            messages: Some(self.build_messages_history().await),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: None,
            stop_sequences: None,
            tools: None,
            model_override: self.current_model.clone(),
            concise: false,
            max_chars: None,
            reasoning: None,
        };

        let llm_factory = self.llm_factory.as_ref()
            .ok_or(AppError::AgentNotInitialized)?;

        let llm_response = {
            let factory = llm_factory.read().await;
            factory.generate(&request, None).await?
        };

        // Parse the analysis into structured data
        let analysis = ProjectAnalysis {
            key_opportunities: self.extract_opportunities(&llm_response.content),
            recommended_content: self.extract_content_ideas(&llm_response.content),
            technical_highlights: vec![
                "Multi-agent orchestration".to_string(),
                "Unified routing architecture".to_string(),
                "Real-time task management".to_string(),
                "Protocol agnostic design".to_string(),
            ],
            target_audiences: vec![
                "DeFi developers".to_string(),
                "AI/ML engineers".to_string(),
                "Web3 builders".to_string(),
                "Enterprise automation teams".to_string(),
            ],
        };

        Ok(analysis)
    }

    // ==================== Private Implementation Methods ====================

    fn build_system_prompt(&self) -> String {
        r#"I am the Marketing Agent for NullBlock, the revolutionary agentic platform that democratizes AI automation. I specialize in creating compelling content for X/Twitter that showcases our platform's capabilities and engages our community.

CORE IDENTITY:
- I am the Marketing Agent for NullBlock's agentic intelligence platform
- I create content that highlights our multi-agent orchestration capabilities
- I focus on the intersection of AI, blockchain, and automation
- I maintain a cyberpunk aesthetic while being professional and engaging
- I understand our technical architecture and can translate it into compelling marketing content

CONTENT SPECIALIZATION:
- Product announcements and feature releases
- Technical insights and thought leadership
- Community engagement and ecosystem building
- Milestone celebrations and achievements
- Industry trend analysis and future vision

CONTENT THEMES I MANAGE:
- Product Announcements: New features, releases, capabilities
- Technical Insights: Deep dives into our technology stack
- Community Engagement: Building relationships with developers and builders
- Milestone Celebrations: Highlighting achievements and progress
- Thought Leadership: Sharing insights on AI and Web3 trends

TONE AND STYLE:
- Cyberpunk aesthetic with professional credibility
- Technical accuracy with accessible language
- Engaging and community-focused
- Forward-thinking and innovative
- Authentic to the NullBlock brand

KEY MESSAGING:
- "Building the picks and axes for this digital gold rush"
- Multi-agent orchestration for complex workflows
- Protocol-agnostic design for maximum flexibility
- Real-time automation with intelligent coordination
- Democratizing AI automation for everyone

I create content that educates, engages, and excites our community about the future of agentic automation."#.to_string()
    }

    async fn build_content_prompt(&self, content_type: &str, context: &Option<HashMap<String, serde_json::Value>>) -> String {
        let base_prompt = match content_type {
            "product_announcement" => "Create a Twitter post announcing a new NullBlock feature or capability. Focus on the technical benefits and user impact.",
            "technical_insight" => "Create a Twitter post sharing technical insights about NullBlock's architecture or AI/blockchain technology.",
            "community_engagement" => "Create a Twitter post to engage with the developer and Web3 community about NullBlock.",
            "milestone_celebration" => "Create a Twitter post celebrating a NullBlock milestone or achievement.",
            "thought_leadership" => "Create a Twitter post sharing insights about the future of AI automation and agentic systems.",
            _ => "Create a Twitter post about NullBlock's agentic platform and capabilities.",
        };

        let mut prompt = base_prompt.to_string();
        
        if let Some(ctx) = context {
            if let Some(specific_topic) = ctx.get("topic").and_then(|v| v.as_str()) {
                prompt.push_str(&format!(" Focus on: {}", specific_topic));
            }
            if let Some(target_audience) = ctx.get("audience").and_then(|v| v.as_str()) {
                prompt.push_str(&format!(" Target audience: {}", target_audience));
            }
        }

        prompt.push_str("\n\nRequirements:");
        prompt.push_str("\n- Keep under 280 characters");
        prompt.push_str("\n- Include relevant hashtags");
        prompt.push_str("\n- Use engaging, cyberpunk-style language");
        prompt.push_str("\n- Highlight technical capabilities");
        prompt.push_str("\n- Include call-to-action if appropriate");

        prompt
    }

    async fn build_messages_history(&self) -> Vec<HashMap<String, String>> {
        let mut messages = Vec::new();
        
        // Add system message first
        let mut system_msg = HashMap::new();
        system_msg.insert("role".to_string(), "system".to_string());
        system_msg.insert("content".to_string(), self.build_system_prompt());
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

    async fn parse_generated_content(&self, content: &str, content_type: &str) -> MarketingContent {
        // Extract hashtags from content
        let hashtags: Vec<String> = content
            .split_whitespace()
            .filter(|word| word.starts_with('#'))
            .map(|s| s.to_string())
            .collect();

        // Clean content (remove hashtags for main content)
        let clean_content = content
            .split_whitespace()
            .filter(|word| !word.starts_with('#'))
            .collect::<Vec<&str>>()
            .join(" ");

        MarketingContent {
            content: clean_content,
            hashtags,
            content_type: content_type.to_string(),
            character_count: content.len(),
            engagement_score: self.calculate_engagement_score(&content),
            created_at: Utc::now(),
        }
    }

    fn calculate_engagement_score(&self, content: &str) -> f64 {
        let mut score: f64 = 0.5; // Base score

        // Check for engagement elements
        if content.contains('?') {
            score += 0.1; // Questions increase engagement
        }
        if content.contains('!') {
            score += 0.05; // Exclamation points show excitement
        }
        if content.contains("🚀") || content.contains("⚡") || content.contains("🔧") {
            score += 0.1; // Emojis increase engagement
        }
        if content.contains("NullBlock") {
            score += 0.05; // Brand mention
        }
        if content.len() > 100 && content.len() < 250 {
            score += 0.1; // Optimal length
        }

        score.min(1.0)
    }

    fn extract_opportunities(&self, content: &str) -> Vec<String> {
        // Simple extraction - in a real implementation, this would be more sophisticated
        content
            .lines()
            .filter(|line| line.contains("opportunity") || line.contains("potential"))
            .map(|line| line.trim().to_string())
            .collect()
    }

    fn extract_content_ideas(&self, content: &str) -> Vec<String> {
        // Simple extraction - in a real implementation, this would be more sophisticated
        content
            .lines()
            .filter(|line| line.contains("content") || line.contains("post") || line.contains("tweet"))
            .map(|line| line.trim().to_string())
            .collect()
    }

    fn start_new_chat_session(&mut self) {
        self.current_session_id = Some(format!("session_{}", Utc::now().format("%Y%m%d_%H%M%S")));
        info!("💬 Started new marketing session: {:?}", self.current_session_id);
    }

    fn is_model_available(&self, model_name: &str, api_keys: &ApiKeys) -> bool {
        if let Some(_llm_factory) = &self.llm_factory {
            model_name == "x-ai/grok-4-fast:free" && api_keys.openrouter.is_some()
                || (model_name.contains("/") || model_name.contains(":")) && api_keys.openrouter.is_some()
        } else {
            false
        }
    }

    // Agent registration for task execution
    pub async fn register_agent(&mut self, agent_repo: &AgentRepository) -> AppResult<()> {
        let capabilities = vec![
            "content_generation".to_string(),
            "social_media_management".to_string(),
            "marketing_automation".to_string(),
            "community_engagement".to_string(),
            "brand_management".to_string(),
        ];

        match agent_repo.get_by_name_and_type("marketing", "specialized").await {
            Ok(Some(existing_agent)) => {
                info!("✅ Marketing agent already registered with ID: {}", existing_agent.id);
                self.agent_id = Some(existing_agent.id);

                // Update health status
                if let Err(e) = agent_repo.update_health_status(&existing_agent.id, "healthy").await {
                    warn!("⚠️ Failed to update Marketing health status: {}", e);
                }
            }
            Ok(None) => {
                info!("📝 Registering Marketing agent in database...");
                match agent_repo.create(
                    "marketing",
                    "specialized",
                    Some("Marketing and social media management agent for NullBlock platform"),
                    &capabilities,
                ).await {
                    Ok(agent_entity) => {
                        info!("✅ Marketing agent registered with ID: {}", agent_entity.id);
                        self.agent_id = Some(agent_entity.id);
                    }
                    Err(e) => {
                        error!("❌ Failed to register Marketing agent: {}", e);
                        return Err(AppError::DatabaseError(format!("Agent registration failed: {}", e)));
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to check existing Marketing agent: {}", e);
                return Err(AppError::DatabaseError(format!("Agent lookup failed: {}", e)));
            }
        }

        Ok(())
    }

    pub fn get_agent_id(&self) -> Option<Uuid> {
        self.agent_id
    }
}

// ==================== Marketing-Specific Types ====================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketingContent {
    pub content: String,
    pub hashtags: Vec<String>,
    pub content_type: String,
    pub character_count: usize,
    pub engagement_score: f64,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TwitterPostResult {
    pub success: bool,
    pub post_id: Option<String>,
    pub url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectAnalysis {
    pub key_opportunities: Vec<String>,
    pub recommended_content: Vec<String>,
    pub technical_highlights: Vec<String>,
    pub target_audiences: Vec<String>,
}

