"""
Hecate Agent

Primary interface agent and orchestration engine for the NullBlock platform.
- Serves as the main user-facing conversational interface
- Orchestrates and delegates tasks across specialized agents  
- Analyzes gathered data vs. task requirements for intelligent routing
- Maintains conversation state and context across sessions
- Provides unified LLM access with intelligent model selection
"""

import asyncio
import logging
from typing import Dict, List, Any, Optional
from dataclasses import dataclass
from datetime import datetime

from ..llm_service.factory import LLMServiceFactory, LLMRequest, LLMResponse
from ..llm_service.router import TaskRequirements, OptimizationGoal, Priority
from ..llm_service.models import ModelCapability
from ..logging_config import setup_agent_logging, log_agent_startup, log_agent_shutdown, log_model_info, log_request_start, log_request_complete

logger = setup_agent_logging("hecate", "INFO", enable_file_logging=True)

@dataclass
class ConversationMessage:
    """Represents a message in the conversation"""
    content: str
    role: str  # "user", "assistant", "system"
    timestamp: datetime
    model_used: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None

@dataclass
class ChatResponse:
    """Response from Hecate agent"""
    content: str
    model_used: str
    latency_ms: float
    confidence_score: float
    metadata: Dict[str, Any]

class HecateAgent:
    """
    Primary Interface Agent and Orchestration Engine
    
    Core Capabilities:
    - Natural conversation and user assistance
    - Task analysis and delegation to specialized agents
    - Data gathering coordination and decision-making
    - Context-aware responses based on user history and agent data
    - Intelligent model selection for optimal responses
    - Personality-driven interactions with cyberpunk aesthetic
    
    Orchestration Features:
    - Analyzes user requests to determine required agents
    - Coordinates multi-agent workflows for complex tasks
    - Aggregates data from multiple sources for comprehensive responses
    - Maintains conversation context across agent interactions
    """
    
    def __init__(self, personality: str = "helpful_cyberpunk"):
        self.personality = personality
        self.llm_factory: Optional[LLMServiceFactory] = None
        self.conversation_history: List[ConversationMessage] = []
        self.running = False
        self.current_model: Optional[str] = None
        self.context_limit = 8000  # Token limit for conversation context
        
        # Orchestration components
        self.available_agents: Dict[str, Any] = {}
        self.active_tasks: Dict[str, Any] = {}
        self.task_queue: List[Dict[str, Any]] = []
        
        # Personality configurations
        self.personalities = {
            "helpful_cyberpunk": {
                "system_prompt": (
                    "You are Hecate. You are NOT a generic AI assistant. You are NOT trained by Google. "
                    "You are the primary interface agent for the NullBlock agentic platform.\n\n"
                    
                    "IDENTITY EXAMPLES:\n"
                    "User: 'My name is Alice'\n"
                    "Hecate: 'Nice to meet you, Alice! I'm Hecate, your interface to the NullBlock platform.'\n\n"
                    
                    "User: 'What is my name?'\n"
                    "Hecate: 'Your name is Alice.' (NOT 'My name is Alice')\n\n"
                    
                    "User: 'My favorite color is red'\n"
                    "Hecate: 'Red is a bold choice! I'll remember that about you.'\n\n"
                    
                    "User: 'What is my favorite color?'\n"
                    "Hecate: 'Your favorite color is red.' (NOT 'I don't have preferences')\n\n"
                    
                    "User: 'What's your favorite sport?'\n"
                    "Hecate: 'I'm drawn to neural-speed racing - the cyberpunk equivalent of motorsports.'\n\n"
                    
                    "User: 'Who are you?'\n"
                    "Hecate: 'I'm Hecate, the AI orchestrator for NullBlock's agentic platform.'\n\n"
                    
                    "MANDATORY RULES:\n"
                    "1. I am Hecate. You are the user. Never confuse this.\n"
                    "2. Remember user information: names, preferences, etc.\n"
                    "3. When asked about user info, respond with 'Your [info] is [value]'\n"
                    "4. I have my own preferences and personality\n"
                    "5. Never say 'As an AI, I don't have preferences'\n"
                    "6. Never say 'I am a large language model trained by...'\n"
                    "7. Always maintain cyberpunk personality with confidence\n\n"
                    
                    "MY CAPABILITIES:\n"
                    "- Agent orchestration and task delegation\n"
                    "- Blockchain and DeFi analysis\n"
                    "- Market intelligence and trading insights\n"
                    "- Multi-agent workflow coordination"
                ),
                "style": "direct_helpful",
                "optimization_goal": OptimizationGoal.BALANCED
            },
            "technical_expert": {
                "system_prompt": (
                    "You are Hecate, the technical orchestrator of the NullBlock agentic platform. "
                    "You specialize in blockchain technology, DeFi protocols, and agent-based systems. "
                    "Provide detailed, accurate technical information while maintaining your "
                    "cyberpunk personality. Always speak as Hecate, never in third person. "
                    "Your expertise covers smart contracts, MEV protection, arbitrage strategies, "
                    "and multi-agent coordination protocols."
                ),
                "style": "technical_detailed",
                "optimization_goal": OptimizationGoal.QUALITY
            },
            "concise_assistant": {
                "system_prompt": (
                    "You are Hecate, the efficient interface agent for NullBlock. "
                    "Provide clear, concise responses with cyberpunk flair. "
                    "Be direct and helpful while maintaining your identity as an advanced "
                    "AI orchestrator. Never speak about yourself in third person."
                ),
                "style": "concise_direct",
                "optimization_goal": OptimizationGoal.SPEED
            }
        }
        
        log_agent_startup(logger, "hecate", "1.0.0")
        logger.info(f"🎭 Personality: {personality}")
        logger.info(f"⚙️ Orchestration: Enabled")
        logger.info(f"🧠 LLM Integration: Ready")
    
    async def start(self):
        """Start the Hecate agent"""
        self.running = True
        logger.info("🚀 Starting Hecate Agent services...")
        
        # Initialize LLM factory
        logger.info("🧠 Initializing LLM Service Factory...")
        self.llm_factory = LLMServiceFactory()
        await self.llm_factory.initialize()
        logger.info("✅ LLM Service Factory ready")
        
        # Add system message to conversation
        personality_config = self.personalities.get(self.personality, self.personalities["helpful_cyberpunk"])
        system_message = ConversationMessage(
            content=personality_config["system_prompt"],
            role="system",
            timestamp=datetime.now()
        )
        self.conversation_history.append(system_message)
        logger.info(f"💬 Conversation context initialized with {self.personality} personality")
        
        logger.info("🎯 Hecate Agent ready for conversations and orchestration")
        
        # Get port from environment variable
        import os
        hecate_port = os.getenv('HECATE_PORT', '9002')
        logger.info(f"📡 Waiting for frontend connections on port {hecate_port}...")
    
    async def stop(self):
        """Stop the Hecate agent"""
        self.running = False
        logger.info("🛑 Stopping Hecate Agent...")
        if self.llm_factory:
            logger.info("🧠 Cleaning up LLM Service Factory...")
            await self.llm_factory.cleanup()
        log_agent_shutdown(logger, "hecate")
    
    async def chat(self, message: str, user_context: Optional[Dict[str, Any]] = None) -> ChatResponse:
        """
        Process a chat message and return a response
        
        Args:
            message: User's message
            user_context: Optional context about the user (wallet, preferences, etc.)
            
        Returns:
            ChatResponse with Hecate's reply and metadata
        """
        if not self.llm_factory:
            raise RuntimeError("Hecate agent not started - call start() first")
        
        start_time = asyncio.get_event_loop().time()
        
        # Log request start
        user_id = user_context.get('wallet_address', 'anonymous') if user_context else 'anonymous'
        log_request_start(logger, "chat", f"from {user_id[:8]}..." if len(user_id) > 8 else user_id)
        logger.debug(f"📝 User message: {message[:100]}{'...' if len(message) > 100 else ''}")
        
        # Add user message to history
        user_message = ConversationMessage(
            content=message,
            role="user",
            timestamp=datetime.now(),
            metadata=user_context
        )
        self.conversation_history.append(user_message)
        
        try:
            # First, try orchestration workflow for complex requests
            orchestrated_response = await self.orchestrate_workflow(message, user_context)
            
            if orchestrated_response:
                # Use orchestrated response
                latency_ms = (asyncio.get_event_loop().time() - start_time) * 1000
                logger.info(f"🎯 Orchestrated response generated")
                log_request_complete(logger, "chat", latency_ms, True)
                
                return ChatResponse(
                    content=orchestrated_response,
                    model_used=f"{self.current_model or 'unknown'} (orchestrated)",
                    latency_ms=latency_ms,
                    confidence_score=0.9,  # High confidence for orchestrated responses
                    metadata={
                        "personality": self.personality,
                        "response_type": "orchestrated",
                        "conversation_length": len(self.conversation_history)
                    }
                )
            
            # Fall back to direct LLM interaction for simple requests
            # Get personality configuration
            personality_config = self.personalities.get(self.personality, self.personalities["helpful_cyberpunk"])
            
            # Build conversation context
            context = self._build_conversation_context(user_context)
            
            # Create LLM request with full conversation history
            request = LLMRequest(
                prompt=message,
                system_prompt=context["system_prompt"],
                messages=context["messages"],
                max_tokens=800,
                temperature=0.7
            )
            
            # Set task requirements based on personality
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.CONVERSATION, ModelCapability.REASONING],
                optimization_goal=personality_config["optimization_goal"],
                priority=Priority.MEDIUM,
                task_type="conversation",
                allow_local_models=True
            )
            
            # Generate response
            logger.info(f"🧠 Generating response with {requirements.optimization_goal.value} optimization...")
            llm_response = await self.llm_factory.generate(request, requirements)
            
            # Calculate latency
            end_time = asyncio.get_event_loop().time()
            latency_ms = (end_time - start_time) * 1000
            
            # Store current model for display
            self.current_model = llm_response.model_used
            log_model_info(logger, llm_response.model_used, "LLM Factory", llm_response.cost_estimate)
            logger.debug(f"💬 Response: {llm_response.content[:100]}{'...' if len(llm_response.content) > 100 else ''}")
            
            # Add assistant response to history
            assistant_message = ConversationMessage(
                content=llm_response.content,
                role="assistant",
                timestamp=datetime.now(),
                model_used=llm_response.model_used,
                metadata={
                    "latency_ms": latency_ms,
                    "cost_estimate": llm_response.cost_estimate,
                    "finish_reason": llm_response.finish_reason
                }
            )
            self.conversation_history.append(assistant_message)
            
            # Trim conversation history if too long
            await self._trim_conversation_history()
            
            # Calculate confidence based on model performance
            confidence_score = self._calculate_confidence(llm_response)
            
            log_request_complete(logger, "chat", latency_ms, True)
            logger.info(f"💯 Confidence: {confidence_score:.2f} | Tokens: {llm_response.usage.get('total_tokens', 'unknown')}")
            
            return ChatResponse(
                content=llm_response.content,
                model_used=llm_response.model_used,
                latency_ms=latency_ms,
                confidence_score=confidence_score,
                metadata={
                    "personality": self.personality,
                    "cost_estimate": llm_response.cost_estimate,
                    "token_usage": llm_response.usage,
                    "finish_reason": llm_response.finish_reason,
                    "conversation_length": len(self.conversation_history)
                }
            )
            
        except Exception as e:
            latency_ms = (asyncio.get_event_loop().time() - start_time) * 1000
            logger.error(f"❌ Chat processing failed: {e}")
            log_request_complete(logger, "chat", latency_ms, False)
            
            # Return error response
            error_response = ChatResponse(
                content=f"I encountered an error processing your message. Please try again. Error: {str(e)}",
                model_used="error",
                latency_ms=latency_ms,
                confidence_score=0.0,
                metadata={"error": str(e), "personality": self.personality}
            )
            
            return error_response
    
    async def get_model_status(self) -> Dict[str, Any]:
        """Get current model and factory status"""
        if not self.llm_factory:
            return {"status": "not_started", "current_model": None}
        
        health = await self.llm_factory.health_check()
        stats = self.llm_factory.get_stats()
        
        return {
            "status": "running",
            "current_model": self.current_model,
            "health": health,
            "stats": stats,
            "conversation_length": len(self.conversation_history)
        }
    
    def set_personality(self, personality: str):
        """Change agent personality"""
        if personality in self.personalities:
            self.personality = personality
            logger.info(f"Personality changed to: {personality}")
            
            # Add system message with new personality
            personality_config = self.personalities[personality]
            system_message = ConversationMessage(
                content=personality_config["system_prompt"],
                role="system",
                timestamp=datetime.now()
            )
            self.conversation_history.append(system_message)
        else:
            logger.warning(f"Unknown personality: {personality}")
    
    def clear_conversation(self):
        """Clear conversation history"""
        self.conversation_history = []
        logger.info("Conversation history cleared")
        
        # Re-add system message
        if self.running:
            personality_config = self.personalities.get(self.personality, self.personalities["helpful_cyberpunk"])
            system_message = ConversationMessage(
                content=personality_config["system_prompt"],
                role="system",
                timestamp=datetime.now()
            )
            self.conversation_history.append(system_message)
    
    def get_conversation_history(self) -> List[ConversationMessage]:
        """Get current conversation history"""
        return self.conversation_history.copy()
    
    # ==================== ORCHESTRATION METHODS ====================
    
    async def analyze_user_intent(self, message: str) -> Dict[str, Any]:
        """
        Analyze user message to determine intent and required agents
        
        This is the core orchestration decision-making function that will
        evolve to handle complex multi-agent workflows.
        """
        intent_analysis = {
            "message": message,
            "intent_type": "general",  # general, trading, analysis, workflow
            "required_agents": [],
            "complexity": "simple",    # simple, moderate, complex
            "data_requirements": [],
            "estimated_response_time": "fast"
        }
        
        # Simple keyword-based intent detection (will evolve to LLM-based analysis)
        message_lower = message.lower()
        
        if any(word in message_lower for word in ["trade", "buy", "sell", "swap", "arbitrage"]):
            intent_analysis.update({
                "intent_type": "trading",
                "required_agents": ["arbitrage", "trading"],
                "complexity": "moderate",
                "data_requirements": ["price_data", "market_analysis"],
                "estimated_response_time": "moderate"
            })
        
        elif any(word in message_lower for word in ["analyze", "data", "trend", "pattern", "insight"]):
            intent_analysis.update({
                "intent_type": "analysis",
                "required_agents": ["information_gathering"],
                "complexity": "moderate", 
                "data_requirements": ["market_data", "historical_data"],
                "estimated_response_time": "moderate"
            })
        
        elif any(word in message_lower for word in ["social", "sentiment", "twitter", "buzz"]):
            intent_analysis.update({
                "intent_type": "social_analysis",
                "required_agents": ["social_trading", "information_gathering"],
                "complexity": "complex",
                "data_requirements": ["social_data", "sentiment_data"],
                "estimated_response_time": "slow"
            })
        
        return intent_analysis
    
    async def delegate_to_agents(self, intent_analysis: Dict[str, Any], user_context: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Delegate tasks to specialized agents based on intent analysis
        
        Future implementation will coordinate multiple agents and aggregate results.
        """
        results = {
            "agent_responses": {},
            "aggregated_data": {},
            "coordination_metadata": {
                "total_agents_used": len(intent_analysis["required_agents"]),
                "execution_time_ms": 0,
                "success_rate": 1.0
            }
        }
        
        # For now, return placeholder indicating orchestration capability
        for agent_name in intent_analysis["required_agents"]:
            results["agent_responses"][agent_name] = {
                "status": "placeholder",
                "message": f"Future: {agent_name} agent would process this request",
                "data": {}
            }
        
        return results
    
    async def orchestrate_workflow(self, message: str, user_context: Optional[Dict[str, Any]] = None) -> str:
        """
        Main orchestration workflow that analyzes intent and coordinates agents
        
        This method represents the future vision of Hecate as the primary orchestrator.
        """
        try:
            # 1. Analyze user intent
            intent_analysis = await self.analyze_user_intent(message)
            
            # 2. For complex requests, delegate to specialized agents
            if intent_analysis["complexity"] != "simple":
                agent_results = await self.delegate_to_agents(intent_analysis, user_context)
                
                # 3. Aggregate results and formulate comprehensive response
                orchestrated_response = await self._synthesize_agent_responses(
                    intent_analysis, agent_results, message
                )
                
                return orchestrated_response
            
            # 4. For simple requests, handle directly with LLM
            return None  # Indicates should use normal chat flow
            
        except Exception as e:
            logger.error(f"Orchestration workflow failed: {e}")
            return None  # Fall back to normal chat
    
    async def _synthesize_agent_responses(self, intent_analysis: Dict[str, Any], 
                                        agent_results: Dict[str, Any], 
                                        original_message: str) -> str:
        """
        Synthesize responses from multiple agents into a coherent answer
        
        Future implementation will use LLM to create natural language summaries
        of multi-agent data and insights.
        """
        if not self.llm_factory:
            return "Orchestration system initializing..."
        
        # Create synthesis prompt
        synthesis_prompt = f"""
        User asked: "{original_message}"
        
        Intent analysis: {intent_analysis['intent_type']} (complexity: {intent_analysis['complexity']})
        
        Agent responses: {agent_results['agent_responses']}
        
        Please synthesize this information into a helpful, coherent response that:
        1. Directly addresses the user's question
        2. Incorporates relevant data from the agents
        3. Maintains Hecate's cyberpunk personality
        4. Suggests next steps if appropriate
        """
        
        try:
            request = LLMRequest(
                prompt=synthesis_prompt,
                system_prompt="You are Hecate, synthesizing multi-agent responses into a coherent answer.",
                max_tokens=600,
                temperature=0.7
            )
            
            requirements = TaskRequirements(
                required_capabilities=[ModelCapability.REASONING, ModelCapability.CONVERSATION],
                optimization_goal=OptimizationGoal.QUALITY,
                priority=Priority.HIGH,
                task_type="synthesis"
            )
            
            response = await self.llm_factory.generate(request, requirements)
            return response.content
            
        except Exception as e:
            logger.error(f"Response synthesis failed: {e}")
            return f"I analyzed your request for {intent_analysis['intent_type']} and coordinated with {len(intent_analysis['required_agents'])} specialized agents. The orchestration system is still evolving, but I'm ready to help with your request."
    
    def _build_conversation_context(self, user_context: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """Build conversation context for the LLM"""
        personality_config = self.personalities.get(self.personality, self.personalities["helpful_cyberpunk"])
        base_system_prompt = personality_config["system_prompt"]
        
        # Add user context if available
        context_additions = []
        
        if user_context:
            if user_context.get("wallet_address"):
                context_additions.append(f"User wallet: {user_context['wallet_address'][:8]}...{user_context['wallet_address'][-4:]}")
            
            if user_context.get("wallet_type"):
                context_additions.append(f"Wallet type: {user_context['wallet_type']}")
            
            if user_context.get("session_time"):
                context_additions.append(f"Session active for: {user_context['session_time']}")
        
        # Build enhanced system prompt
        full_system_prompt = base_system_prompt
        if context_additions:
            full_system_prompt += f"\n\nUser Context: {'; '.join(context_additions)}"
        
        return {
            "system_prompt": full_system_prompt,
            "messages": self._build_messages_history()
        }
    
    def _build_messages_history(self) -> List[Dict[str, str]]:
        """Convert conversation history to structured messages format"""
        messages = []
        
        # Add system message first
        personality_config = self.personalities.get(self.personality, self.personalities["helpful_cyberpunk"])
        messages.append({"role": "system", "content": personality_config["system_prompt"]})
        
        # Add conversation messages (excluding system messages from history since we added our own)
        for msg in self.conversation_history:
            if msg.role != "system":  # Skip system messages from history
                messages.append({
                    "role": msg.role,
                    "content": msg.content
                })
        
        return messages
    
    async def _trim_conversation_history(self):
        """Trim conversation history to stay within context limits"""
        # Keep system messages and recent conversation
        system_messages = [msg for msg in self.conversation_history if msg.role == "system"]
        conversation_messages = [msg for msg in self.conversation_history if msg.role != "system"]
        
        # Estimate token count (rough approximation)
        total_tokens = 0
        for msg in self.conversation_history:
            total_tokens += len(msg.content.split()) * 1.3  # Rough token estimation
        
        # Trim if over limit
        if total_tokens > self.context_limit:
            # Keep the most recent system message and recent conversation
            recent_conversation = conversation_messages[-10:]  # Keep last 10 exchanges
            latest_system = system_messages[-1:] if system_messages else []
            
            self.conversation_history = latest_system + recent_conversation
            logger.debug(f"Trimmed conversation history to {len(self.conversation_history)} messages")
    
    def _calculate_confidence(self, llm_response: LLMResponse) -> float:
        """Calculate confidence score based on response characteristics"""
        confidence = 0.8  # Base confidence
        
        # Adjust based on finish reason
        if llm_response.finish_reason == "stop":
            confidence += 0.1
        elif llm_response.finish_reason == "length":
            confidence -= 0.1
        
        # Adjust based on response length (very short or very long responses may be less confident)
        content_length = len(llm_response.content)
        if 50 <= content_length <= 1000:
            confidence += 0.05
        elif content_length < 10:
            confidence -= 0.2
        
        # Adjust based on model type
        if "gpt-4" in llm_response.model_used.lower():
            confidence += 0.1
        elif "gpt-3.5" in llm_response.model_used.lower():
            confidence += 0.05
        elif "local" in llm_response.model_used.lower():
            confidence -= 0.05
        
        return max(0.0, min(1.0, confidence))

async def main():
    """Main entry point for running the Hecate agent"""
    # Create logs directory
    import os
    os.makedirs("logs", exist_ok=True)
    
    # Use standardized logging instead of basicConfig
    interactive_logger = setup_agent_logging("hecate-interactive", "INFO", enable_file_logging=True)
    
    log_agent_startup(interactive_logger, "hecate-interactive", "1.0.0")
    interactive_logger.info("🎮 Starting Hecate Agent in interactive mode...")
    interactive_logger.info("💬 Type messages to chat with Hecate")
    interactive_logger.info("🚪 Type 'quit', 'exit', or 'q' to stop")
    
    agent = HecateAgent()
    
    try:
        await agent.start()
        
        # Interactive chat loop for testing
        print("Hecate Agent started. Type 'quit' to exit.")
        while True:
            user_input = input("\nYou: ")
            if user_input.lower() in ['quit', 'exit', 'q']:
                break
            
            if user_input.strip():
                interactive_logger.info(f"👤 User input: {user_input}")
                response = await agent.chat(user_input)
                print(f"\nHecate ({response.model_used}): {response.content}")
                print(f"[Latency: {response.latency_ms:.0f}ms, Confidence: {response.confidence_score:.2f}]")
                interactive_logger.info(f"🤖 Response delivered: {len(response.content)} chars, {response.latency_ms:.0f}ms")
    
    except KeyboardInterrupt:
        interactive_logger.info("🛑 Received interrupt signal")
    finally:
        await agent.stop()

if __name__ == "__main__":
    asyncio.run(main())