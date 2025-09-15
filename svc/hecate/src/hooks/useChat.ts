import { useState, useRef } from 'react';

interface ChatMessage {
  id: string;
  timestamp: Date;
  sender: 'user' | 'hecate';
  message: string;
  type?: string;
  model_used?: string;
  metadata?: any;
}

export const useChat = (publicKey: string | null) => {
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [chatInput, setChatInput] = useState('');
  const [isProcessingChat, setIsProcessingChat] = useState(false);
  const [chatAutoScroll, setChatAutoScroll] = useState(true);
  const [isUserScrolling, setIsUserScrolling] = useState(false);

  const chatEndRef = useRef<HTMLDivElement>(null);
  const chatMessagesRef = useRef<HTMLDivElement>(null);
  const chatInputRef = useRef<HTMLInputElement>(null);
  const userScrollTimeoutRef = useRef<NodeJS.Timeout | null>(null);

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
      const { hecateAgent } = await import('../common/services/hecate-agent');

      const connected = await hecateAgent.connect();
      if (!connected) {
        throw new Error('Failed to connect to Hecate agent');
      }

      console.log('🔄 Sending message to Hecate agent, thinking state should be active...');
      const response = await hecateAgent.sendMessage(message, {
        wallet_address: publicKey || undefined,
        wallet_type: localStorage.getItem('walletType') || undefined,
        session_time: new Date().toISOString()
      });
      console.log('✅ Received response from Hecate, changing from thinking state...');

      const hecateMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        timestamp: new Date(),
        sender: 'hecate',
        message: response.content,
        type: 'text',
        model_used: response.model_used,
        metadata: response.metadata
      };

      setChatMessages((prev) => [...prev, hecateMessage]);

      setIsProcessingChat(false);
      if (response.confidence_score > 0.8) {
        setNulleyeState('success');
      } else if (response.metadata?.finish_reason === 'question') {
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

        if (errorMsg.includes('lm studio') || errorMsg.includes('localhost:1234')) {
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
        sender: 'hecate',
        message: userFriendlyMessage,
        type: 'error',
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

  const handleChatInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { value } = e.target;
    setChatInput(value);
  };

  const handleChatScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const container = e.currentTarget;
    const scrollTop = container.scrollTop;
    const scrollHeight = container.scrollHeight;
    const clientHeight = container.clientHeight;

    const isNearBottom = scrollHeight - scrollTop - clientHeight <= 50;

    if (!isNearBottom && !isUserScrolling) {
      setIsUserScrolling(true);
      setChatAutoScroll(false);

      if (userScrollTimeoutRef.current) {
        clearTimeout(userScrollTimeoutRef.current);
      }

      userScrollTimeoutRef.current = setTimeout(() => {
        setIsUserScrolling(false);
        setChatAutoScroll(true);
      }, 3000);
    } else if (isNearBottom && isUserScrolling) {
      setIsUserScrolling(false);
      setChatAutoScroll(true);

      if (userScrollTimeoutRef.current) {
        clearTimeout(userScrollTimeoutRef.current);
      }
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
    chatEndRef,
    chatMessagesRef,
    chatInputRef,
    userScrollTimeoutRef,
    handleChatSubmit,
    handleChatInputChange,
    handleChatScroll
  };
};