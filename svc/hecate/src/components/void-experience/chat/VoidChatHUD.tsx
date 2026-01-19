import React, { useState, useRef, useEffect, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { agentService } from '../../../common/services/agent-service';
import { hecateAgent } from '../../../common/services/hecate-agent';
import MarkdownRenderer from '../../common/MarkdownRenderer';
import CommandDropdown from './CommandDropdown';
import { useCommands, type SlashCommand } from '../../../hooks/useCommands';
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
  hasOverlappingPanels?: boolean;
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
  hasOverlappingPanels = false,
}) => {
  // Format model name for display (extract short name from full path)
  const formatModelName = (model: string | null): string => {
    if (!model) {
      return 'READY';
    }

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

  // Slash command state
  const [showCommandDropdown, setShowCommandDropdown] = useState(false);
  const [commandSelectedIndex, setCommandSelectedIndex] = useState(0);
  const {
    filterCommands,
    isToolQuery,
    getHelpText,
    getToolListText,
    getMcpStatusText,
    mcpTools,
  } = useCommands();

  // Compute filtered commands based on current input
  const filteredCommands = input.startsWith('/') ? filterCommands(input) : [];

  // Handle input changes for slash command detection
  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    setInput(value);

    // Show dropdown when typing "/" at start
    if (value.startsWith('/')) {
      setShowCommandDropdown(true);
      setCommandSelectedIndex(0);
    } else {
      setShowCommandDropdown(false);
    }
  }, []);

  // Handle command selection
  const handleCommandSelect = useCallback((command: SlashCommand) => {
    setShowCommandDropdown(false);
    setInput('');

    // Execute built-in commands immediately
    if (command.action === 'execute') {
      let responseText = '';

      switch (command.name) {
        case '/help':
          responseText = getHelpText();
          break;
        case '/list-tools':
          responseText = getToolListText();
          break;
        case '/mcp':
          responseText = getMcpStatusText();
          break;
        case '/clear':
          setMessages([]);
          return;
        case '/status':
          responseText = `## Agent Status\n\n**Active Agent**: ${activeAgent.toUpperCase()}\n**Health**: ${agentHealthStatus}\n**MCP Tools**: ${mcpTools.length} available`;
          break;
        default:
          responseText = `Command ${command.name} not implemented yet.`;
      }

      // Add command response as agent message
      const agentMsg: VoidMessage = {
        id: `cmd-${Date.now()}`,
        text: responseText,
        sender: 'agent',
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, agentMsg]);
      setShowHistory(true);
    } else if (command.action === 'tools') {
      // Show tool list
      const toolListText = getToolListText();
      const agentMsg: VoidMessage = {
        id: `tools-${Date.now()}`,
        text: toolListText,
        sender: 'agent',
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, agentMsg]);
      setShowHistory(true);
    } else {
      // Insert command as message to send to agent
      const commandMsg = `Use the ${command.name.replace('/', '')} tool to help me.`;
      setInput(commandMsg);
      inputRef.current?.focus();
    }
  }, [activeAgent, agentHealthStatus, mcpTools.length, getHelpText, getToolListText, getMcpStatusText]);

  const inputRef = useRef<HTMLTextAreaElement>(null);
  const historyRef = useRef<HTMLDivElement>(null);
  const pendingMessageRef = useRef<{ message: string; msgId: string } | null>(null);
  const tooltipTimerRef = useRef<NodeJS.Timeout | null>(null);
  const hasShownWelcomeRef = useRef(false);
  const lastPanelStateRef = useRef(false);
  const showHistoryRef = useRef(false);

  // Mark as hydrated on client mount (SSR-safe)
  // Chat is ephemeral - no persistence across page refresh
  useEffect(() => {
    setIsHydrated(true);
    // Clear backend conversation on fresh session (paranoid cleanup)
    agentService.clearConversation('hecate').catch(() => {});
  }, []);

  // Handle the actual API call after charging/firing animation
  const executeTransmission = useCallback(async () => {
    const pending = pendingMessageRef.current;

    if (!pending) {
      return;
    }

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

        setMessages((prev) => [...prev, agentMsg]);

        // Start tooltip timer only if:
        // 1. User hasn't acknowledged the first message feature yet
        // 2. History panel is currently closed (user didn't see the message)
        if (!hasAcknowledgedFirst && !showHistoryRef.current) {
          // Clear any existing timer
          if (tooltipTimerRef.current) {
            clearTimeout(tooltipTimerRef.current);
          }

          // Show tooltip after 10 seconds if history is still closed
          tooltipTimerRef.current = setTimeout(() => {
            // Double-check history is still closed when timer fires
            if (!showHistoryRef.current) {
              setShowTooltip(true);
            }
          }, 10000);
        }
      } else {
        const errorMsg: VoidMessage = {
          id: `error-${Date.now()}`,
          text: 'The void remains silent...',
          sender: 'agent',
          timestamp: new Date(),
        };

        setMessages((prev) => [...prev, errorMsg]);
      }
    } catch (error) {
      console.error('Void chat error:', error);
      const errorMsg: VoidMessage = {
        id: `error-${Date.now()}`,
        text: 'A disturbance in the void...',
        sender: 'agent',
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, errorMsg]);
    } finally {
      setIsProcessing(false);
      setEnergyState('idle');
      pendingMessageRef.current = null;
    }
  }, [onAgentResponseReceived, hasAcknowledgedFirst, activeAgent]);

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();

      if (!input.trim() || isProcessing || energyState !== 'idle') {
        return;
      }

      const userMessage = input.trim();

      // Close command dropdown
      setShowCommandDropdown(false);

      // Check if this is a natural language tool query
      if (isToolQuery(userMessage)) {
        setInput('');

        // Add user message
        const userMsg: VoidMessage = {
          id: `user-${Date.now()}`,
          text: userMessage,
          sender: 'user',
          timestamp: new Date(),
        };
        setMessages((prev) => [...prev, userMsg]);

        // Respond with tool list
        const toolResponse = `I have access to **${mcpTools.length} MCP tools** across various services. Here's a summary:\n\n${getToolListText()}\n\n**Tip**: Type \`/\` to see available slash commands, or ask me to use any specific tool.`;

        const agentMsg: VoidMessage = {
          id: `tools-${Date.now()}`,
          text: toolResponse,
          sender: 'agent',
          timestamp: new Date(),
        };
        setMessages((prev) => [...prev, agentMsg]);
        setShowHistory(true);
        return;
      }

      setInput('');

      // Add user message immediately
      const userMsg: VoidMessage = {
        id: `user-${Date.now()}`,
        text: userMessage,
        sender: 'user',
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, userMsg]);

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
    },
    [input, isProcessing, energyState, hasInteracted, onFirstMessage, executeTransmission, isToolQuery, mcpTools.length, getToolListText],
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    // Handle command dropdown navigation
    if (showCommandDropdown && filteredCommands.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setCommandSelectedIndex((prev) =>
          prev < filteredCommands.length - 1 ? prev + 1 : 0
        );
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setCommandSelectedIndex((prev) =>
          prev > 0 ? prev - 1 : filteredCommands.length - 1
        );
        return;
      }
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleCommandSelect(filteredCommands[commandSelectedIndex]);
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        setShowCommandDropdown(false);
        return;
      }
      if (e.key === 'Tab') {
        e.preventDefault();
        // Tab autocompletes the selected command
        setInput(filteredCommands[commandSelectedIndex].name + ' ');
        return;
      }
    }

    // Close dropdown on Escape even if empty
    if (e.key === 'Escape' && showCommandDropdown) {
      e.preventDefault();
      setShowCommandDropdown(false);
      return;
    }

    // Normal Enter to submit
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e as unknown as React.FormEvent);
    }
  };

  // Auto-resize textarea
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.style.height = 'auto';
      inputRef.current.style.height = `${Math.min(inputRef.current.scrollHeight, 120)}px`;
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
  useEffect(
    () => () => {
      if (tooltipTimerRef.current) {
        clearTimeout(tooltipTimerRef.current);
      }
    },
    [],
  );

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

  // Sync with external showHistory control (from Hecate panel toggle)
  // Open AND close history based on external control
  useEffect(() => {
    if (externalShowHistory !== undefined) {
      setShowHistory(externalShowHistory);
    }
  }, [externalShowHistory]);

  // Keep showHistoryRef in sync for use in callbacks/timers
  useEffect(() => {
    showHistoryRef.current = showHistory;
  }, [showHistory]);

  // Show Hecate welcome/return message when Studio opens
  useEffect(() => {
    const panelJustOpened = externalShowHistory === true && lastPanelStateRef.current === false;

    lastPanelStateRef.current = externalShowHistory || false;

    if (panelJustOpened && activeAgent === 'hecate') {
      // Use functional update to check messages without dependency
      setMessages((prev) => {
        const hasUserMessages = prev.some((m) => m.sender === 'user');

        if (!hasUserMessages && !hasShownWelcomeRef.current) {
          // First time opening with no conversation - show welcome
          hasShownWelcomeRef.current = true;
          const welcomeText =
            HECATE_WELCOME_MESSAGES[Math.floor(Math.random() * HECATE_WELCOME_MESSAGES.length)];

          return [
            ...prev,
            {
              id: `hecate-welcome-${Date.now()}`,
              text: welcomeText,
              sender: 'agent' as const,
              timestamp: new Date(),
            },
          ];
        } else if (hasUserMessages) {
          // Returning mid-conversation - show return message
          const returnText =
            HECATE_RETURN_MESSAGES[Math.floor(Math.random() * HECATE_RETURN_MESSAGES.length)];

          return [
            ...prev,
            {
              id: `hecate-return-${Date.now()}`,
              text: returnText,
              sender: 'agent' as const,
              timestamp: new Date(),
            },
          ];
        }

        return prev;
      });
    }
  }, [externalShowHistory, activeAgent]);

  if (!isActive) {
    return null;
  }

  // Use portal to render at body level, escaping VoidExperience's stacking context
  const chatContent = (
    <div className={styles.voidChatContainer}>
      {/* Input bar */}
      <div className={styles.voidInputBar}>
        {/* Chat History Popup */}
        {showHistory && (
          <div className={`${styles.historyPopup} ${hasOverlappingPanels ? styles.elevated : ''}`}>
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
                  <span className={styles.healthWarning} title="API keys required">
                    ⚠️
                  </span>
                )}
              </div>
              <button
                className={styles.historyClose}
                onClick={() => setShowHistory(false)}
                aria-label="Close history"
              >
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                >
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
                          {msg.timestamp.toLocaleTimeString([], {
                            hour: '2-digit',
                            minute: '2-digit',
                          })}
                        </span>
                      </div>
                      {msg.isTaskResult && (
                        <div className={styles.taskResultHeader}>
                          <div className={styles.taskResultBadge}>
                            <span className={styles.taskIcon}>✅</span>
                            <span className={styles.taskLabel}>Task Result</span>
                            {msg.taskName && (
                              <span className={styles.taskName}>"{msg.taskName}"</span>
                            )}
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
          <div
            className={`${styles.inputContainer} ${energyState === 'charging' ? styles.charging : ''} ${energyState === 'firing' ? styles.firing : ''} ${energyState === 'processing' ? styles.processing : ''} ${glowActive ? styles.receiving : ''}`}
          >
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
                style={{
                  transform: showHistory ? 'rotate(0deg)' : 'rotate(180deg)',
                  transition: 'transform 0.3s ease',
                }}
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
            {/* Slash command dropdown */}
            {showCommandDropdown && (
              <CommandDropdown
                commands={filteredCommands}
                selectedIndex={commandSelectedIndex}
                onSelect={handleCommandSelect}
                onClose={() => setShowCommandDropdown(false)}
                query={input}
              />
            )}
            <textarea
              ref={inputRef}
              value={input}
              onChange={handleInputChange}
              onKeyDown={handleKeyDown}
              placeholder={
                agentHealthStatus === 'unhealthy'
                  ? '⚠️ Configure API keys first...'
                  : energyState === 'processing'
                    ? `Awaiting ${activeAgent} response...`
                    : 'Chat with Hecate... (type / for commands)'
              }
              className={styles.voidInput}
              disabled={energyState !== 'idle' || agentHealthStatus === 'unhealthy'}
              rows={1}
            />
            <button
              type="submit"
              className={styles.sendButton}
              disabled={
                energyState !== 'idle' || !input.trim() || agentHealthStatus === 'unhealthy'
              }
              aria-label="Send message"
            >
              ➤
            </button>
          </div>
        </form>
      </div>
    </div>
  );

  // Render via portal to escape VoidExperience's stacking context (z-index: 1)
  // This allows the chat to appear above HUD content (z-index: 1002)
  if (typeof document !== 'undefined') {
    return createPortal(chatContent, document.body);
  }

  return chatContent;
};

export default VoidChatHUD;
