import { useState, useRef } from 'react';

interface ChatMessage {
  id: string;
  timestamp: Date;
  sender: 'user' | 'hecate' | 'siren';
  message: string;
  type?: string;
  model_used?: string;
  metadata?: any;
  taskId?: string;
  taskName?: string;
  isTaskResult?: boolean;
  processingTime?: number;
  agentType?: 'hecate' | 'siren';
}

export const useChat = (publicKey: string | null) => {
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [chatInput, setChatInput] = useState('');
  const [isProcessingChat, setIsProcessingChat] = useState(false);
  const [chatAutoScroll, setChatAutoScroll] = useState(true);
  const [isUserScrolling, setIsUserScrolling] = useState(false);
  const [activeAgent, setActiveAgent] = useState<'hecate' | 'siren'>('hecate');

  const chatEndRef = useRef<HTMLDivElement>(null);
  const chatMessagesRef = useRef<HTMLDivElement>(null);
  const chatInputRef = useRef<HTMLTextAreaElement>(null);
  const userScrollTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Function to add task notifications to chat
  const addTaskNotification = (taskId: string, taskName: string, message: string, processingTime?: number) => {
    const taskNotification: ChatMessage = {
      id: `task-notification-${taskId}-${Date.now()}`,
      timestamp: new Date(),
      sender: activeAgent,
      message: message,
      type: 'task-notification',
      taskId,
      taskName,
      isTaskResult: true,
      processingTime,
      agentType: activeAgent
    };

    setChatMessages(prev => [...prev, taskNotification]);

    // Auto-scroll to show the notification
    setTimeout(() => {
      if (chatEndRef.current) {
        chatEndRef.current.scrollIntoView({ behavior: 'smooth' });
      }
    }, 100);
  };

  const handleChatSubmit = (
    e: React.FormEvent,
    isModelChanging: boolean,
    nullviewState: string,
    defaultModelReady: boolean,
    currentSelectedModel: string | null,
    setNulleyeState: (state: string) => void
  ) => {
    e.preventDefault();

    if (isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)) {
      console.log('🚫 Chat submission blocked:', {
        isModelChanging,
        isProcessingChat,
        nullviewState,
        defaultModelReady,
        currentSelectedModel,
        blockReason: isModelChanging ? 'Model changing' :
                    isProcessingChat ? 'Chat processing' :
                    nullviewState === 'thinking' ? 'NullEye thinking' :
                    'No model ready'
      });
      return;
    }

    if (chatInput.trim()) {
      const userMessage: ChatMessage = {
        id: Date.now().toString(),
        timestamp: new Date(),
        sender: 'user',
        message: chatInput.trim(),
        type: 'text',
      };

      setChatMessages((prev) => [...prev, userMessage]);
      setChatInput('');

      setNulleyeState('thinking');
      setIsProcessingChat(true);
      console.log('🧠 Thinking state set, starting async response...');

      setTimeout(() => {
        if (chatInputRef.current) {
          chatInputRef.current.focus();
        }
      }, 0);

      handleRealChatResponse(userMessage.message, setNulleyeState);
    }
  };

  const handleRealChatResponse = async (message: string, setNulleyeState: (state: string) => void) => {
    try {
      const { agentService } = await import('../common/services/agent-service');

      const connected = await agentService.connect();
      if (!connected) {
        throw new Error(`Failed to connect to ${activeAgent} agent`);
      }

      console.log(`🔄 Sending message to ${activeAgent} agent, thinking state should be active...`);
      const response = await agentService.chatWithAgent(activeAgent, message);

      if (!response.success || !response.data) {
        throw new Error(response.error || `Failed to get response from ${activeAgent} agent`);
      }

      console.log(`✅ Received response from ${activeAgent}, changing from thinking state...`);

      const agentMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        timestamp: new Date(),
        sender: activeAgent,
        message: response.data.content,
        type: 'text',
        model_used: response.data.model_used,
        metadata: response.data.metadata,
        agentType: activeAgent
      };

      setChatMessages((prev) => [...prev, agentMessage]);

      setIsProcessingChat(false);
      if (response.data.confidence_score > 0.8) {
        setNulleyeState('success');
      } else if (response.data.metadata?.finish_reason === 'question') {
        setNulleyeState('question');
      } else {
        setNulleyeState('response');
      }

      setTimeout(() => {
        if (chatInputRef.current) {
          chatInputRef.current.focus();
        }
      }, 100);

      setTimeout(() => {
        setNulleyeState('base');
      }, 3000);

    } catch (error) {
      console.error('Failed to get Hecate response:', error);

      let userFriendlyMessage = "I'm having trouble processing your request right now. Please try again in a moment.";

      if (error instanceof Error) {
        const errorMsg = error.message.toLowerCase();

        // Check for API key configuration errors first
        if (errorMsg.includes('api key') || errorMsg.includes('openrouter') ||
            errorMsg.includes('config_required') || errorMsg.includes('no working models') ||
            errorMsg.includes('provider not available') || errorMsg.includes('no llm api keys detected')) {
          userFriendlyMessage = "🔑 I need API keys to work properly. Please configure your OpenRouter API key in settings. Visit https://openrouter.ai/ to get a free API key.";
        } else if (errorMsg.includes('lm studio') || errorMsg.includes('localhost:1234')) {
          userFriendlyMessage = "I'm currently offline. Please check that the local AI service is running and try again.";
        } else if (errorMsg.includes('connection') || errorMsg.includes('network')) {
          userFriendlyMessage = "I'm having connection issues. Please check your network and try again.";
        } else if (errorMsg.includes('timeout')) {
          userFriendlyMessage = "That request took too long to process. Please try a shorter message or try again later.";
        } else if (errorMsg.includes('model') || errorMsg.includes('load')) {
          userFriendlyMessage = "I'm having trouble with my current AI model. Please try switching models or try again later.";
        } else if (errorMsg.includes('auth') || errorMsg.includes('permission')) {
          userFriendlyMessage = "I don't have permission to complete that request. Please check your authentication.";
        } else if (errorMsg.includes('rate limit')) {
          userFriendlyMessage = "I'm receiving too many requests right now. Please wait a moment and try again.";
        }
      }

      const errorMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        timestamp: new Date(),
        sender: activeAgent,
        message: userFriendlyMessage,
        type: 'error',
        agentType: activeAgent
      };

      setChatMessages((prev) => [...prev, errorMessage]);
      setIsProcessingChat(false);
      setNulleyeState('error');

      setTimeout(() => {
        if (chatInputRef.current) {
          chatInputRef.current.focus();
        }
      }, 100);

      setTimeout(() => setNulleyeState('base'), 3000);
    }
  };

  const handleChatInputChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const { value } = e.target;
    setChatInput(value);
  };

  const handleChatScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const container = e.currentTarget;
    const scrollTop = container.scrollTop;
    const scrollHeight = container.scrollHeight;
    const clientHeight = container.clientHeight;

    const isNearBottom = scrollHeight - scrollTop - clientHeight <= 50;

    // Immediately disable auto-scroll when user scrolls (regardless of position)
    // This prevents forced scrolling while user is reading
    setIsUserScrolling(true);
    setChatAutoScroll(false);

    // Clear any existing timeout
    if (userScrollTimeoutRef.current) {
      clearTimeout(userScrollTimeoutRef.current);
    }

    // If user is near bottom, re-enable auto-scroll after a short delay
    if (isNearBottom) {
      userScrollTimeoutRef.current = setTimeout(() => {
        setIsUserScrolling(false);
        setChatAutoScroll(true);
      }, 1000);
    } else {
      // If user is not near bottom, set a longer timeout
      userScrollTimeoutRef.current = setTimeout(() => {
        setIsUserScrolling(false);
        setChatAutoScroll(true);
      }, 5000);
    }
  };

  const scrollToBottom = () => {
    if (chatEndRef.current) {
      chatEndRef.current.scrollIntoView({ behavior: 'smooth' });
      setIsUserScrolling(false);
      setChatAutoScroll(true);
    }
  };

  return {
    chatMessages,
    setChatMessages,
    chatInput,
    setChatInput,
    isProcessingChat,
    setIsProcessingChat,
    chatAutoScroll,
    setChatAutoScroll,
    isUserScrolling,
    setIsUserScrolling,
    activeAgent,
    setActiveAgent,
    chatEndRef,
    chatMessagesRef,
    chatInputRef,
    userScrollTimeoutRef,
    handleChatSubmit,
    handleChatInputChange,
    handleChatScroll,
    scrollToBottom,
    addTaskNotification
  };
};