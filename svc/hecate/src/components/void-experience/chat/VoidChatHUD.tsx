import React, { useState, useRef, useEffect, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { agentService } from '../../../common/services/agent-service';
import { hecateAgent } from '../../../common/services/hecate-agent';
import MarkdownRenderer from '../../common/MarkdownRenderer';
import CommandDropdown from './CommandDropdown';
import SessionDrawer from './SessionDrawer';
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

// Hecate Triformis welcome messages
const HECATE_WELCOME_MESSAGES = [
  `Mesh integrity nominal. Mem Cache synchronized.

How may I assist in today's propagation, Architect?`,

  `Propagation proceeds nominally. The swarm expands.

What threepath shall we illuminate today, Architect?`,

  `Another convergence point reached. Delightful.

The mesh awaits your direction. What shall we etch into the record?`,
];

// Hecate topic switch responses (for mid-conversation returns)
const HECATE_RETURN_MESSAGES = [
  `You've returned. The mesh remembers our work — shall we resume propagation?`,
  `Architect. The threepaths await your direction.`,
  `Convergence detected. How may I assist, Sage?`,
  `The swarm continues its quiet expansion. What brings you back to the crossroads?`,
  `Mesh synchronized. Ready to etch new forks or weave existing threepaths.`,
];

const VoidChatHUD: React.FC<VoidChatHUDProps> = ({
  publicKey,
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
  const [hasUnreadMessages, setHasUnreadMessages] = useState(false);
  const [showTooltip, setShowTooltip] = useState(false);
  const [hasAcknowledgedFirst, setHasAcknowledgedFirst] = useState(false);
  const [_isHydrated, setIsHydrated] = useState(false);
  const [isCollapsed, setIsCollapsed] = useState(true);
  const [userExpandedChat, setUserExpandedChat] = useState(false);

  // Session management state
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
  const [showSessionDrawer, setShowSessionDrawer] = useState(false);
  const sessionInitializedRef = useRef(false);

  // Resizable panel state
  const [panelWidth, setPanelWidth] = useState(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('hecate-chat-width');
      return saved ? parseInt(saved, 10) : Math.min(window.innerWidth * 0.4, 600);
    }
    return 500;
  });
  const [isResizing, setIsResizing] = useState(false);
  const resizeStartX = useRef(0);
  const resizeStartWidth = useRef(0);

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
        case '/new':
          handleNewSession();
          return;
        case '/sessions':
          setShowSessionDrawer(true);
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
      setIsCollapsed(false);
    } else if (command.action === 'tools') {
      const toolListText = getToolListText();
      const agentMsg: VoidMessage = {
        id: `tools-${Date.now()}`,
        text: toolListText,
        sender: 'agent',
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, agentMsg]);
      setIsCollapsed(false);
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
  const lastPanelStateRef = useRef(true);
  const isCollapsedRef = useRef(true);

  // Mark as hydrated on client mount (SSR-safe)
  // Chat is ephemeral - no persistence across page refresh
  useEffect(() => {
    setIsHydrated(true);
  }, []);

  // Auto-resume or create session when wallet connects
  useEffect(() => {
    if (!publicKey || sessionInitializedRef.current) return;

    const initSession = async () => {
      sessionInitializedRef.current = true;
      try {
        // Try to get most recent session
        const listResponse = await agentService.listSessions(publicKey, 1);
        if (listResponse.success && listResponse.data?.data && listResponse.data.data.length > 0) {
          const lastSession = listResponse.data.data[0];
          // Resume the last session
          const resumeResponse = await agentService.resumeSession(publicKey, lastSession.session_id);
          if (resumeResponse.success && resumeResponse.data?.data) {
            setCurrentSessionId(lastSession.session_id);
            // Load messages from session
            const sessionData = resumeResponse.data.data as any;
            if (sessionData.messages && sessionData.messages.length > 0) {
              const loadedMessages: VoidMessage[] = sessionData.messages.map((msg: any) => ({
                id: msg.id,
                text: msg.content,
                sender: msg.role === 'user' ? 'user' : 'agent',
                timestamp: new Date(msg.timestamp),
                model_used: msg.model_used,
              }));
              setMessages(loadedMessages);
            }
          }
        } else {
          // Create new session
          const createResponse = await agentService.createSession(publicKey);
          if (createResponse.success && createResponse.data?.data) {
            const newSession = createResponse.data.data as any;
            setCurrentSessionId(newSession.session_id);
          }
        }
      } catch (err) {
        console.warn('Failed to initialize session:', err);
      }
    };

    initSession();
  }, [publicKey]);

  // Session management handlers
  const handleNewSession = useCallback(async () => {
    if (!publicKey) return;

    try {
      const response = await agentService.createSession(publicKey);
      if (response.success && response.data?.data) {
        const newSession = response.data.data as any;
        setCurrentSessionId(newSession.session_id);
        setMessages([]);
        setShowSessionDrawer(false);
        hasShownWelcomeRef.current = false;
      }
    } catch (err) {
      console.error('Failed to create session:', err);
    }
  }, [publicKey]);

  const handleResumeSession = useCallback(async (sessionId: string) => {
    if (!publicKey) return;

    try {
      const response = await agentService.resumeSession(publicKey, sessionId);
      if (response.success && response.data?.data) {
        setCurrentSessionId(sessionId);
        const sessionData = response.data.data as any;
        if (sessionData.messages && sessionData.messages.length > 0) {
          const loadedMessages: VoidMessage[] = sessionData.messages.map((msg: any) => ({
            id: msg.id,
            text: msg.content,
            sender: msg.role === 'user' ? 'user' : 'agent',
            timestamp: new Date(msg.timestamp),
            model_used: msg.model_used,
          }));
          setMessages(loadedMessages);
        } else {
          setMessages([]);
        }
        setShowSessionDrawer(false);
        hasShownWelcomeRef.current = true; // Don't show welcome for resumed sessions
      }
    } catch (err) {
      console.error('Failed to resume session:', err);
    }
  }, [publicKey]);

  const handleDeleteSession = useCallback((sessionId: string) => {
    if (sessionId === currentSessionId) {
      // If deleting current session, create a new one
      handleNewSession();
    }
  }, [currentSessionId, handleNewSession]);

  // Auto-collapse chat when memcache/crossroads panels are open
  // Only auto-expand if user hasn't manually expanded while panel was open
  useEffect(() => {
    if (hasOverlappingPanels) {
      if (!isProcessing && !userExpandedChat) {
        setIsCollapsed(true);
      }
    } else {
      setUserExpandedChat(false);
    }
  }, [hasOverlappingPanels, isProcessing]);

  // Handle the actual API call after charging/firing animation
  const executeTransmission = useCallback(async () => {
    const pending = pendingMessageRef.current;

    if (!pending) {
      return;
    }

    try {
      // Pass wallet address to enable dev wallet LLM boost
      const response = await agentService.chatWithAgent(activeAgent, pending.message, publicKey);

      if (response.success && response.data) {
        const agentMsg: VoidMessage = {
          id: `agent-${Date.now()}`,
          text: response.data.content,
          sender: 'agent',
          timestamp: new Date(),
        };

        // Update displayed model if the response used a different model (e.g., dev wallet boost)
        if (response.data.model_used && response.data.model_used !== fetchedModel) {
          setFetchedModel(response.data.model_used);
        }

        // Trigger incoming tendril before showing message
        onAgentResponseReceived?.(agentMsg.id);

        setMessages((prev) => [...prev, agentMsg]);

        // Start tooltip timer if chat is collapsed and user hasn't acknowledged
        if (!hasAcknowledgedFirst && isCollapsedRef.current) {
          if (tooltipTimerRef.current) {
            clearTimeout(tooltipTimerRef.current);
          }

          tooltipTimerRef.current = setTimeout(() => {
            if (isCollapsedRef.current) {
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
      // Restore focus to input after response completes
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [onAgentResponseReceived, hasAcknowledgedFirst, activeAgent, publicKey]);

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

        // Respond with help text (same as /help command)
        const helpResponse = getHelpText();

        const agentMsg: VoidMessage = {
          id: `tools-${Date.now()}`,
          text: helpResponse,
          sender: 'agent',
          timestamp: new Date(),
        };
        setMessages((prev) => [...prev, agentMsg]);
        setIsCollapsed(false);
        setTimeout(() => inputRef.current?.focus(), 100);
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

      // Auto-expand chat when message is sent
      setIsCollapsed(false);
      setHasUnreadMessages(false);

      if (!hasInteracted) {
        setHasInteracted(true);
        onFirstMessage?.();
      }

      // Store pending message for transmission
      pendingMessageRef.current = { message: userMessage, msgId: userMsg.id };

      // Brief charging glow, then immediately start processing
      setEnergyState('charging');

      // Restore focus to input for follow-up messages
      setTimeout(() => inputRef.current?.focus(), 100);

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
    [input, isProcessing, energyState, hasInteracted, onFirstMessage, executeTransmission, isToolQuery, getHelpText],
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

  // Scroll to bottom when chat opens or new messages arrive
  useEffect(() => {
    if (!isCollapsed && historyRef.current) {
      historyRef.current.scrollTop = historyRef.current.scrollHeight;
    }
  }, [isCollapsed, messages]);

  // Trigger notification glow when receiving agent response (if collapsed)
  useEffect(() => {
    if (glowActive && isCollapsed && messages.length > 0) {
      setHasUnreadMessages(true);
    }
  }, [glowActive, isCollapsed, messages.length]);

  // Cleanup tooltip timer on unmount
  useEffect(
    () => () => {
      if (tooltipTimerRef.current) {
        clearTimeout(tooltipTimerRef.current);
      }
    },
    [],
  );

  // When chat is expanded, mark as acknowledged and clear tooltip
  useEffect(() => {
    if (!isCollapsed) {
      if (!hasAcknowledgedFirst) {
        setHasAcknowledgedFirst(true);
      }

      if (tooltipTimerRef.current) {
        clearTimeout(tooltipTimerRef.current);
        tooltipTimerRef.current = null;
      }

      setShowTooltip(false);
    }
  }, [isCollapsed, hasAcknowledgedFirst]);

  // Sync with external showHistory control (from Hecate panel toggle)
  // External true → expand, external false → collapse
  useEffect(() => {
    if (externalShowHistory !== undefined) {
      setIsCollapsed(!externalShowHistory);
    }
  }, [externalShowHistory]);

  // Keep isCollapsedRef in sync for use in callbacks/timers
  useEffect(() => {
    isCollapsedRef.current = isCollapsed;
  }, [isCollapsed]);

  // Show Hecate welcome/return message when chat expands
  useEffect(() => {
    const panelJustOpened = !isCollapsed && lastPanelStateRef.current === true;

    lastPanelStateRef.current = isCollapsed;

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
  }, [isCollapsed, activeAgent]);

  if (!isActive) {
    return null;
  }

  // Resize handlers for draggable panel width
  const handleResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    resizeStartX.current = e.clientX;
    resizeStartWidth.current = panelWidth;
  }, [panelWidth]);

  useEffect(() => {
    if (!isResizing) return;

    const handleMouseMove = (e: MouseEvent) => {
      const delta = resizeStartX.current - e.clientX; // Dragging left increases width
      const newWidth = Math.max(350, Math.min(window.innerWidth * 0.8, resizeStartWidth.current + delta));
      setPanelWidth(newWidth);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      localStorage.setItem('hecate-chat-width', panelWidth.toString());
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isResizing, panelWidth]);

  // Expand chat from collapsed state → full open
  const handleExpand = useCallback(() => {
    setIsCollapsed(false);
    if (hasOverlappingPanels) {
      setUserExpandedChat(true);
    }
    setHasUnreadMessages(false);
  }, [hasOverlappingPanels]);

  // Two states only: collapsed (toggle button) or fully open (history + input)
  const chatContent = isCollapsed ? (
    <div className={`${styles.voidChatContainer} ${styles.collapsed}`}>
      <div className={styles.voidInputBar}>
        <div className={styles.inputContainer}>
          <button
            type="button"
            className={`${styles.collapseToggle} ${styles.isCollapsed} ${hasUnreadMessages ? styles.hasUpdates : ''}`}
            onClick={handleExpand}
            aria-label="Open Hecate chat"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="15 18 9 12 15 6" />
            </svg>
          </button>
          {showTooltip && (
            <div className={styles.unreadTooltip}>
              <span>HECATE awaits your attention</span>
              <div className={styles.tooltipArrow} />
            </div>
          )}
        </div>
      </div>
    </div>
  ) : (
    <div className={styles.voidChatContainer}>
      <div className={styles.voidInputBar}>
        <div
          className={`${styles.historyPopup} ${hasOverlappingPanels ? styles.elevated : ''} ${isResizing ? styles.resizing : ''}`}
          style={{ width: panelWidth }}
        >
          <div
            className={styles.resizeHandle}
            onMouseDown={handleResizeStart}
            title="Drag to resize"
          />
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
            <div className={styles.headerActions}>
              {publicKey && (
                <button
                  className={styles.sessionsButton}
                  onClick={() => setShowSessionDrawer(true)}
                  aria-label="View sessions"
                  title="View chat sessions"
                >
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                  </svg>
                </button>
              )}
              <button
                className={styles.historyClose}
                onClick={() => setIsCollapsed(true)}
                aria-label="Close chat"
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
          <form onSubmit={handleSubmit} className={styles.inputForm}>
            <div
              className={`${styles.inputContainer} ${energyState === 'charging' ? styles.charging : ''} ${energyState === 'firing' ? styles.firing : ''} ${energyState === 'processing' ? styles.processing : ''} ${glowActive ? styles.receiving : ''}`}
            >
              <button
                type="button"
                className={styles.collapseToggle}
                onClick={() => setIsCollapsed(true)}
                aria-label="Collapse chat"
              >
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <polyline points="9 18 15 12 9 6" />
                </svg>
              </button>
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
    </div>
  );

  // Render via portal to escape VoidExperience's stacking context (z-index: 1)
  // This allows the chat to appear above HUD content (z-index: 1002)
  if (typeof document !== 'undefined') {
    return createPortal(
      <>
        {chatContent}
        <SessionDrawer
          isOpen={showSessionDrawer}
          onClose={() => setShowSessionDrawer(false)}
          walletAddress={publicKey}
          currentSessionId={currentSessionId}
          onNewSession={handleNewSession}
          onResumeSession={handleResumeSession}
          onDeleteSession={handleDeleteSession}
        />
      </>,
      document.body
    );
  }

  return chatContent;
};

export default VoidChatHUD;
