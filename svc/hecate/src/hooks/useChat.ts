import { useState, useRef, useEffect } from 'react';

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
  content?: {
    type: 'text' | 'image' | 'mixed';
    text?: string;
    imageIds?: string[];
  };
}

interface ImageData {
  url: string;
  alt?: string;
  caption?: string;
}

// Helper function to detect if a request is for image generation
const isImageGenerationRequest = (message: string): boolean => {
  const imageKeywords = [
    'logo',
    'image',
    'picture',
    'photo',
    'draw',
    'create',
    'generate',
    'design',
    'visual',
    'graphic',
    'illustration',
    'artwork',
    'sketch',
    'render',
    'show me',
    'make me',
    'give me',
    'create a',
    'design a',
    'draw a',
  ];

  const lowerMessage = message.toLowerCase();

  return imageKeywords.some((keyword) => lowerMessage.includes(keyword));
};

// Helper function to parse content and extract images
const parseContentForImages = (
  content: string,
): { content: string; images: { url: string; alt?: string; caption?: string }[] } => {
  const images: { url: string; alt?: string; caption?: string }[] = [];

  // Regex patterns to detect various image formats
  const markdownImagePattern = /!\[([^\]]*)\]\(([^)]+)\)/g;
  const base64ImagePattern = /data:image\/([a-zA-Z]*);base64,([^"\s)]+)/g;
  const urlImagePattern = /https?:\/\/[^\s)]+\.(jpg|jpeg|png|gif|webp|svg)(\?[^\s)]*)?/gi;

  // Extract markdown images
  let match;
  while ((match = markdownImagePattern.exec(content)) !== null) {
    const alt = match[1] || 'Generated image';
    const url = match[2];

    images.push({ url, alt, caption: undefined });
  }

  // Extract standalone base64 images (not in markdown)
  const base64Matches = content.match(base64ImagePattern);

  if (base64Matches) {
    base64Matches.forEach((url) => {
      if (!images.some((img) => img.url === url)) {
        images.push({ url, alt: 'Generated image', caption: undefined });
      }
    });
  }

  // Extract standalone URL images (not in markdown)
  const urlMatches = content.match(urlImagePattern);

  if (urlMatches) {
    urlMatches.forEach((url) => {
      if (!images.some((img) => img.url === url)) {
        images.push({ url, alt: 'Generated image', caption: undefined });
      }
    });
  }

  // IMPORTANT: Keep original content with markdown images intact for MarkdownRenderer
  // The images array is just for metadata and fallback display
  return { content, images };
};

export const useChat = (_publicKey: string | null) => {
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
  const imageStorageRef = useRef<Map<string, ImageData>>(new Map());
  const previousPublicKeyRef = useRef<string | null>(null);

  // Clear chat when wallet changes (different user logged in)
  useEffect(() => {
    const currentKey = _publicKey;
    const previousKey = previousPublicKeyRef.current;

    // Only clear if publicKey actually changed (not on initial mount)
    if (previousKey !== null && currentKey !== previousKey) {
      setChatMessages([]);
      imageStorageRef.current.clear();
      setChatInput('');
      setIsProcessingChat(false);
    }

    previousPublicKeyRef.current = currentKey;
  }, [_publicKey]);

  // Function to add task notifications to chat
  const addTaskNotification = (
    taskId: string,
    taskName: string,
    message: string,
    processingTime?: number,
  ) => {
    const taskNotification: ChatMessage = {
      id: `task-notification-${taskId}-${Date.now()}`,
      timestamp: new Date(),
      sender: activeAgent,
      message,
      type: 'task-notification',
      taskId,
      taskName,
      isTaskResult: true,
      processingTime,
      agentType: activeAgent,
    };

    setChatMessages((prev) => [...prev, taskNotification]);

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
    setNulleyeState: (state: string) => void,
  ) => {
    e.preventDefault();

    if (
      isModelChanging ||
      isProcessingChat ||
      nullviewState === 'thinking' ||
      (!defaultModelReady && !currentSelectedModel)
    ) {
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

      setTimeout(() => {
        if (chatInputRef.current) {
          chatInputRef.current.focus();
        }
      }, 0);

      handleRealChatResponse(userMessage.message, setNulleyeState);
    }
  };

  const handleRealChatResponse = async (
    message: string,
    setNulleyeState: (state: string) => void,
  ) => {
    try {
      const { agentService } = await import('../common/services/agent-service');

      const connected = await agentService.connect();

      if (!connected) {
        throw new Error(`Failed to connect to ${activeAgent} agent`);
      }

      // Check if this is an image generation request
      const isImageRequest = isImageGenerationRequest(message);

      const response = await agentService.chatWithAgent(activeAgent, message);

      if (!response.success || !response.data) {
        throw new Error(response.error || `Failed to get response from ${activeAgent} agent`);
      }

      // Parse content to detect images
      const { content, images } = parseContentForImages(response.data.content);

      const messageId = (Date.now() + 1).toString();
      const imageIds: string[] = [];

      if (images.length > 0) {
        images.forEach((image, index) => {
          const imageId = `${messageId}-img-${index}`;

          imageStorageRef.current.set(imageId, image);
          imageIds.push(imageId);
        });
      }

      const agentMessage: ChatMessage = {
        id: messageId,
        timestamp: new Date(),
        sender: activeAgent,
        message: content,
        type: images.length > 0 ? 'mixed' : 'text',
        model_used: response.data.model_used,
        metadata: response.data.metadata,
        agentType: activeAgent,
        content: {
          type: images.length > 0 ? (content ? 'mixed' : 'image') : 'text',
          text: content,
          imageIds,
        },
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

      let userFriendlyMessage =
        "I'm having trouble processing your request right now. Please try again in a moment.";

      if (error instanceof Error) {
        const errorMsg = error.message.toLowerCase();

        // Check for model not available errors first (most specific)
        if (
          errorMsg.includes('not available') ||
          errorMsg.includes('not found') ||
          errorMsg.includes('not currently available')
        ) {
          userFriendlyMessage =
            '🚫 The selected AI model is not currently available. Please select a different model from the model selection dropdown. We recommend trying DeepSeek Chat v3.1 or Dolphin Mistral 24B (both free).';
        }
        // Check for API key configuration errors
        else if (
          errorMsg.includes('api key') ||
          errorMsg.includes('openrouter') ||
          errorMsg.includes('config_required') ||
          errorMsg.includes('no working models') ||
          errorMsg.includes('provider not available') ||
          errorMsg.includes('no llm api keys detected')
        ) {
          userFriendlyMessage =
            '🔑 I need API keys to work properly. Please configure your OpenRouter API key in settings. Visit https://openrouter.ai/ to get a free API key.';
        } else if (errorMsg.includes('lm studio') || errorMsg.includes('localhost:1234')) {
          userFriendlyMessage =
            "I'm currently offline. Please check that the local AI service is running and try again.";
        } else if (errorMsg.includes('connection') || errorMsg.includes('network')) {
          userFriendlyMessage =
            "I'm having connection issues. Please check your network and try again.";
        } else if (errorMsg.includes('timeout')) {
          userFriendlyMessage =
            'That request took too long to process. Please try a shorter message or try again later.';
        } else if (errorMsg.includes('model') || errorMsg.includes('load')) {
          userFriendlyMessage =
            "I'm having trouble with my current AI model. Please try switching models or try again later.";
        } else if (errorMsg.includes('auth') || errorMsg.includes('permission')) {
          userFriendlyMessage =
            "I don't have permission to complete that request. Please check your authentication.";
        } else if (errorMsg.includes('rate limit')) {
          userFriendlyMessage =
            "I'm receiving too many requests right now. Please wait a moment and try again.";
        }
      }

      const errorMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        timestamp: new Date(),
        sender: activeAgent,
        message: userFriendlyMessage,
        type: 'error',
        agentType: activeAgent,
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
    const { scrollTop } = container;
    const { scrollHeight } = container;
    const { clientHeight } = container;

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

  const getImagesForMessage = (messageId: string): ImageData[] => {
    const message = chatMessages.find((msg) => msg.id === messageId);

    if (!message?.content?.imageIds) {
      return [];
    }

    return message.content.imageIds
      .map((id) => imageStorageRef.current.get(id))
      .filter((img): img is ImageData => img !== undefined);
  };

  // Clear all chat messages and stored images (for logout/user switch)
  const clearMessages = () => {
    setChatMessages([]);
    imageStorageRef.current.clear();
    setChatInput('');
    setIsProcessingChat(false);
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
    addTaskNotification,
    getImagesForMessage,
    clearMessages,
  };
};
