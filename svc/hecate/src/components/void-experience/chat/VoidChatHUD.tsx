import React, { useState, useRef, useEffect, useCallback } from 'react';
import { agentService } from '../../../common/services/agent-service';
import { hecateAgent } from '../../../common/services/hecate-agent';
import MarkdownRenderer from '../../common/MarkdownRenderer';
import styles from './voidChat.module.scss';

interface ImageData {
  url: string;
  alt?: string;
  caption?: string;
}

interface VoidMessage {
  id: string;
  text: string;
  sender: 'user' | 'agent';
  timestamp: Date;
  isTaskResult?: boolean;
  taskName?: string;
  taskId?: string;
  processingTime?: number;
  model_used?: string;
}

interface VoidChatHUDProps {
  publicKey: string | null;
  isActive?: boolean;
  onFirstMessage?: () => void;
  onAgentResponseReceived?: (messageId: string) => void;
  glowActive?: boolean;
  currentModel?: string | null;
  activeAgent?: 'hecate' | 'siren';
  setActiveAgent?: (agent: 'hecate' | 'siren') => void;
  agentHealthStatus?: 'healthy' | 'unhealthy' | 'unknown';
  getImagesForMessage?: (messageId: string) => ImageData[];
  showHistory?: boolean;
}

// Energy state for transmission animation
type EnergyState = 'idle' | 'charging' | 'firing' | 'processing';

// Hecate welcome messages (slight variations)
const HECATE_WELCOME_MESSAGES = [
  `Welcome, visitor.

I am Hecate, your companion at the edge.

The Studio is a personal space to capture, reflect, and build upon interactions that occur within the Crossroads.

Show me an agent, a tool, a workflow... and we will break it down, rebuild it, make it yours.`,

  `Welcome to the Studio, visitor.

I am Hecate — guide at the boundary between intention and action.

This is your space to examine, deconstruct, and reimagine what you discover in the Crossroads.

Bring me an agent, a protocol, a piece of the mesh... together, we'll make it your own.`,

  `Visitor, you've arrived.

I am Hecate, keeper of the Studio.

Here we transform curiosity into creation. The Crossroads shows you what exists — the Studio helps you make it yours.

An agent, a tool, a workflow — show me what caught your eye.`,
];

// Hecate topic switch responses (for mid-conversation returns)
const HECATE_RETURN_MESSAGES = [
  `You've returned. The Studio awaits your next inquiry.`,
  `Back at the edge, visitor. What shall we examine?`,
  `The Studio opens once more. Continue where we left off, or bring something new.`,
  `Welcome back. The void remembers our work — shall we resume?`,
  `You return to the Studio. What draws your attention now?`,
];

const CHAT_STORAGE_KEY = 'nullblock_void_chat_history';
const MAX_PERSISTED_MESSAGES = 20;

const VoidChatHUD: React.FC<VoidChatHUDProps> = ({
  publicKey: _publicKey,
  isActive = true,
  onFirstMessage,
  onAgentResponseReceived,
  glowActive = false,
  currentModel: externalModel = null,
  activeAgent = 'hecate',
  setActiveAgent,
  agentHealthStatus = 'unknown',
  getImagesForMessage,
  showHistory: externalShowHistory,
}) => {
  // Format model name for display (extract short name from full path)
  const formatModelName = (model: string | null): string => {
    if (!model) return 'READY';
    return model.split('/').pop()?.split(':')[0]?.toUpperCase() || 'MODEL';
  };

  const [fetchedModel, setFetchedModel] = useState<string | null>(null);
  const currentModel = externalModel || fetchedModel;

  // Fetch current model from hecate agent on mount
  useEffect(() => {
    if (!externalModel) {
      const fetchModel = async () => {
        try {
          const connected = await hecateAgent.connect();
          if (connected) {
            const status = await hecateAgent.getModelStatus();
            if (status.current_model) {
              setFetchedModel(status.current_model);
            }
          }
        } catch (err) {
          console.warn('Failed to fetch model info:', err);
        }
      };
      fetchModel();
    }
  }, [externalModel]);

  const [input, setInput] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [energyState, setEnergyState] = useState<EnergyState>('idle');
  const [messages, setMessages] = useState<VoidMessage[]>([]);
  const [hasInteracted, setHasInteracted] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const [hasUnreadMessages, setHasUnreadMessages] = useState(false);
  const [showTooltip, setShowTooltip] = useState(false);
  const [hasAcknowledgedFirst, setHasAcknowledgedFirst] = useState(false);
  const [isHydrated, setIsHydrated] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const historyRef = useRef<HTMLDivElement>(null);
  const pendingMessageRef = useRef<{ message: string; msgId: string } | null>(null);
  const tooltipTimerRef = useRef<NodeJS.Timeout | null>(null);
  const hasShownWelcomeRef = useRef(false);
  const lastPanelStateRef = useRef(false);

  // Load persisted messages from localStorage on client mount (SSR-safe)
  useEffect(() => {
    try {
      const stored = localStorage.getItem(CHAT_STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored);
        const restoredMessages = parsed.map((msg: any) => ({
          ...msg,
          timestamp: new Date(msg.timestamp),
        }));
        setMessages(restoredMessages);
        // If we have user messages, mark as interacted
        if (parsed.some((msg: any) => msg.sender === 'user')) {
          setHasInteracted(true);
        }
        // If we have any messages, welcome was already shown
        if (parsed.length > 0) {
          hasShownWelcomeRef.current = true;
        }
      }
    } catch (e) {
      console.warn('Failed to load chat history:', e);
    }
    setIsHydrated(true);
  }, []);

  // Handle the actual API call after charging/firing animation
  const executeTransmission = useCallback(async () => {
    const pending = pendingMessageRef.current;
    if (!pending) return;

    try {
      const response = await agentService.chatWithAgent(activeAgent, pending.message);

      if (response.success && response.data) {
        const agentMsg: VoidMessage = {
          id: `agent-${Date.now()}`,
          text: response.data.content,
          sender: 'agent',
          timestamp: new Date(),
        };

        // Trigger incoming tendril before showing message
        onAgentResponseReceived?.(agentMsg.id);

        setMessages(prev => [...prev, agentMsg]);

        // Start tooltip timer if user hasn't acknowledged first message yet
        if (!hasAcknowledgedFirst) {
          // Clear any existing timer
          if (tooltipTimerRef.current) {
            clearTimeout(tooltipTimerRef.current);
          }
          // Show tooltip after 10 seconds if history still closed
          tooltipTimerRef.current = setTimeout(() => {
            setShowTooltip(true);
          }, 10000);
        }
      } else {
        const errorMsg: VoidMessage = {
          id: `error-${Date.now()}`,
          text: 'The void remains silent...',
          sender: 'agent',
          timestamp: new Date(),
        };
        setMessages(prev => [...prev, errorMsg]);
      }
    } catch (error) {
      console.error('Void chat error:', error);
      const errorMsg: VoidMessage = {
        id: `error-${Date.now()}`,
        text: 'A disturbance in the void...',
        sender: 'agent',
        timestamp: new Date(),
      };
      setMessages(prev => [...prev, errorMsg]);
    } finally {
      setIsProcessing(false);
      setEnergyState('idle');
      pendingMessageRef.current = null;
    }
  }, [onAgentResponseReceived, hasAcknowledgedFirst, activeAgent]);

  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();

    if (!input.trim() || isProcessing || energyState !== 'idle') return;

    const userMessage = input.trim();
    setInput('');

    // Add user message immediately
    const userMsg: VoidMessage = {
      id: `user-${Date.now()}`,
      text: userMessage,
      sender: 'user',
      timestamp: new Date(),
    };
    setMessages(prev => [...prev, userMsg]);

    // Auto-open chat history when message is sent
    setShowHistory(true);
    setHasUnreadMessages(false);

    if (!hasInteracted) {
      setHasInteracted(true);
      onFirstMessage?.();
    }

    // Store pending message for transmission
    pendingMessageRef.current = { message: userMessage, msgId: userMsg.id };

    // Brief charging glow, then immediately start processing
    setEnergyState('charging');

    // Quick transition to firing then processing (reduced delay)
    setTimeout(() => {
      setEnergyState('firing');

      setTimeout(() => {
        setIsProcessing(true);
        setEnergyState('processing');
        executeTransmission();
      }, 150); // Reduced from 300ms
    }, 400); // Reduced from 800ms
  }, [input, isProcessing, energyState, hasInteracted, onFirstMessage, executeTransmission]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e as unknown as React.FormEvent);
    }
  };

  // Auto-resize textarea
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.style.height = 'auto';
      inputRef.current.style.height = Math.min(inputRef.current.scrollHeight, 120) + 'px';
    }
  }, [input]);

  // Scroll to bottom when history opens or new messages arrive
  useEffect(() => {
    if (showHistory && historyRef.current) {
      historyRef.current.scrollTop = historyRef.current.scrollHeight;
    }
  }, [showHistory, messages]);

  // Trigger notification glow when receiving agent response (if history is closed)
  useEffect(() => {
    if (glowActive && !showHistory && messages.length > 0) {
      setHasUnreadMessages(true);
    }
  }, [glowActive, showHistory, messages.length]);

  // Cleanup tooltip timer on unmount
  useEffect(() => {
    return () => {
      if (tooltipTimerRef.current) {
        clearTimeout(tooltipTimerRef.current);
      }
    };
  }, []);

  // When history is opened, mark as acknowledged and clear tooltip
  useEffect(() => {
    if (showHistory) {
      // User is viewing history, so they've acknowledged messages
      if (!hasAcknowledgedFirst) {
        setHasAcknowledgedFirst(true);
      }
      // Clear any pending tooltip timer
      if (tooltipTimerRef.current) {
        clearTimeout(tooltipTimerRef.current);
        tooltipTimerRef.current = null;
      }
      setShowTooltip(false);
    }
  }, [showHistory, hasAcknowledgedFirst]);

  // Persist messages to localStorage when they change (only after hydration)
  useEffect(() => {
    if (!isHydrated) return; // Don't persist until we've loaded from storage
    if (messages.length > 0) {
      try {
        // Keep only the last N messages to avoid bloating localStorage
        const toStore = messages.slice(-MAX_PERSISTED_MESSAGES);
        localStorage.setItem(CHAT_STORAGE_KEY, JSON.stringify(toStore));
      } catch (e) {
        console.warn('Failed to persist chat history:', e);
      }
    }
  }, [messages, isHydrated]);

  // Sync with external showHistory control (from Hecate panel toggle)
  // Open AND close history based on external control
  useEffect(() => {
    if (externalShowHistory !== undefined) {
      setShowHistory(externalShowHistory);
    }
  }, [externalShowHistory]);

  // Show Hecate welcome/return message when Studio opens
  useEffect(() => {
    const panelJustOpened = externalShowHistory === true && lastPanelStateRef.current === false;
    lastPanelStateRef.current = externalShowHistory || false;

    if (panelJustOpened && activeAgent === 'hecate') {
      // Use functional update to check messages without dependency
      setMessages(prev => {
        const hasUserMessages = prev.some(m => m.sender === 'user');

        if (!hasUserMessages && !hasShownWelcomeRef.current) {
          // First time opening with no conversation - show welcome
          hasShownWelcomeRef.current = true;
          const welcomeText = HECATE_WELCOME_MESSAGES[Math.floor(Math.random() * HECATE_WELCOME_MESSAGES.length)];
          return [...prev, {
            id: `hecate-welcome-${Date.now()}`,
            text: welcomeText,
            sender: 'agent' as const,
            timestamp: new Date(),
          }];
        } else if (hasUserMessages) {
          // Returning mid-conversation - show return message
          const returnText = HECATE_RETURN_MESSAGES[Math.floor(Math.random() * HECATE_RETURN_MESSAGES.length)];
          return [...prev, {
            id: `hecate-return-${Date.now()}`,
            text: returnText,
            sender: 'agent' as const,
            timestamp: new Date(),
          }];
        }
        return prev;
      });
    }
  }, [externalShowHistory, activeAgent]);

  if (!isActive) return null;

  return (
    <div className={styles.voidChatContainer}>
      {/* Input bar */}
      <div className={styles.voidInputBar}>
        {/* Chat History Popup */}
        {showHistory && (
          <div className={styles.historyPopup}>
            <div className={styles.historyHeader}>
              <div className={styles.historyTitleContainer}>
                <span
                  className={styles.historyAgentName}
                  onClick={() => {
                    if (setActiveAgent) {
                      setActiveAgent(activeAgent === 'hecate' ? 'siren' : 'hecate');
                    }
                  }}
                  title={`Click to switch to ${activeAgent === 'hecate' ? 'Siren' : 'Hecate'}`}
                >
                  {activeAgent.toUpperCase()}
                </span>
                <span className={styles.historyModelName}>:{formatModelName(currentModel)}</span>
                {agentHealthStatus === 'unhealthy' && (
                  <span className={styles.healthWarning} title="API keys required">⚠️</span>
                )}
              </div>
              <button
                className={styles.historyClose}
                onClick={() => setShowHistory(false)}
                aria-label="Close history"
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M18 6L6 18M6 6l12 12" />
                </svg>
              </button>
            </div>
            <div className={styles.historyContent} ref={historyRef}>
              {messages.length === 0 ? (
                <div className={styles.historyEmpty}>No transmissions yet...</div>
              ) : (
                messages.map((msg) => {
                  const images = getImagesForMessage ? getImagesForMessage(msg.id) : [];
                  return (
                    <div
                      key={msg.id}
                      className={`${styles.historyMessage} ${msg.sender === 'user' ? styles.historyUser : styles.historyAgent} ${msg.isTaskResult ? styles.historyTaskResult : ''}`}
                    >
                      <div className={styles.historyMeta}>
                        <span className={styles.historySender}>
                          {msg.sender === 'user' ? 'You' : activeAgent.toUpperCase()}
                        </span>
                        <span className={styles.historyTime}>
                          {msg.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                        </span>
                      </div>
                      {msg.isTaskResult && (
                        <div className={styles.taskResultHeader}>
                          <div className={styles.taskResultBadge}>
                            <span className={styles.taskIcon}>✅</span>
                            <span className={styles.taskLabel}>Task Result</span>
                            {msg.taskName && <span className={styles.taskName}>"{msg.taskName}"</span>}
                          </div>
                          {msg.processingTime && (
                            <span className={styles.processingTime}>⏱️ {msg.processingTime}ms</span>
                          )}
                        </div>
                      )}
                      <div className={styles.historyText}>
                        <MarkdownRenderer content={msg.text} images={images} />
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          </div>
        )}
        <form onSubmit={handleSubmit} className={styles.inputForm}>
          <div className={`${styles.inputContainer} ${energyState === 'charging' ? styles.charging : ''} ${energyState === 'firing' ? styles.firing : ''} ${energyState === 'processing' ? styles.processing : ''} ${glowActive ? styles.receiving : ''}`}>
            {/* History toggle button */}
            <button
              type="button"
              className={`${styles.historyToggle} ${showHistory ? styles.historyActive : ''} ${messages.length === 0 ? styles.historyDisabled : ''} ${hasUnreadMessages && !showHistory ? styles.historyNotification : ''}`}
              onClick={() => {
                if (messages.length > 0) {
                  const newShowHistory = !showHistory;
                  setShowHistory(newShowHistory);
                  if (newShowHistory) {
                    setHasUnreadMessages(false);
                    // Clear tooltip timer and hide tooltip
                    if (tooltipTimerRef.current) {
                      clearTimeout(tooltipTimerRef.current);
                      tooltipTimerRef.current = null;
                    }
                    setShowTooltip(false);
                    // Mark that user has acknowledged first message
                    if (!hasAcknowledgedFirst) {
                      setHasAcknowledgedFirst(true);
                    }
                  }
                }
              }}
              aria-label="Toggle chat history"
              disabled={messages.length === 0}
            >
              <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
                style={{ transform: showHistory ? 'rotate(0deg)' : 'rotate(180deg)', transition: 'transform 0.3s ease' }}
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            {/* Tooltip for unread message reminder */}
            {showTooltip && !showHistory && (
              <div className={styles.unreadTooltip}>
                <span>HECATE awaits your attention</span>
                <div className={styles.tooltipArrow} />
              </div>
            )}
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={
                agentHealthStatus === 'unhealthy'
                  ? '⚠️ Configure API keys first...'
                  : energyState === 'processing'
                    ? `Awaiting ${activeAgent} response...`
                    : 'Chat with interface...'
              }
              className={styles.voidInput}
              disabled={energyState !== 'idle' || agentHealthStatus === 'unhealthy'}
              rows={1}
            />
            <button
              type="submit"
              className={styles.sendButton}
              disabled={energyState !== 'idle' || !input.trim() || agentHealthStatus === 'unhealthy'}
              aria-label="Send message"
            >
              ➤
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default VoidChatHUD;
