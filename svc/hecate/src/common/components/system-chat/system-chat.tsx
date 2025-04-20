import React, { useState, useRef, useEffect } from 'react';
import styles from './system-chat.module.scss';
import envelopeIcon from '../../../assets/images/tiny-envelope_17541.png';

interface ChatMessage {
  id: number;
  text: string;
  type: 'message' | 'alert' | 'critical' | 'update' | 'action' | 'user' | 'assistant';
  action?: () => void;
  actionText?: string;
  metadata?: {
    memoryCard?: {
      behavior?: Record<string, any>;
      features?: string[];
    };
    walletHealth?: {
      riskScore?: number;
      activeTokens?: string[];
    };
  };
}

interface SystemChatProps {
  messages: ChatMessage[];
  isEchoActive?: boolean;
  onUserInput?: (input: string) => void;
  currentRoom?: string;
  onRoomChange?: (room: string) => void;
  memoryCard?: {
    userBehavior: Record<string, any>;
    features: string[];
    eventLog: any[];
  };
  walletHealth?: {
    balance: number;
    riskScore: number;
    activeTokens: string[];
  };
  isCollapsed?: boolean;
  onCollapsedChange?: (collapsed: boolean) => void;
  isDigitizing?: boolean;
  theme?: 'null' | 'light';
}

const SystemChat: React.FC<SystemChatProps> = ({ 
  messages, 
  isEchoActive = false, 
  onUserInput,
  currentRoom = '/logs',
  onRoomChange,
  memoryCard,
  walletHealth,
  isCollapsed = true,
  onCollapsedChange,
  isDigitizing = false,
  theme = 'light'
}) => {
  const [input, setInput] = useState('');
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [activeDropdown, setActiveDropdown] = useState<'room' | 'theme' | null>(null);
  const [lastSeenMessageId, setLastSeenMessageId] = useState(0);
  const [isProcessing, setIsProcessing] = useState(false);
  const [showInbox, setShowInbox] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const chatRooms = [
    { id: 'logs', name: '/logs', available: true },
    { id: 'memory', name: '/memory', available: false },
    { id: 'health', name: '/health', available: false },
    { id: 'reality', name: '/reality', available: false }
  ];

  const themes = [
    { id: 'null', name: 'NULL', description: 'Minimalist dark interface', available: true },
    { id: 'light', name: 'NULL-LIGHT', description: 'Minimalist light interface', available: true }
  ];

  // Update collapsed state when prop changes
  useEffect(() => {
    if (onCollapsedChange) {
      onCollapsedChange(isCollapsed);
    }
  }, [isCollapsed]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setActiveDropdown(null);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Check if there are any new pending action messages
  const hasNewActionMessages = messages.some(msg => 
    msg.type === 'action' && msg.id > lastSeenMessageId
  );

  // Update last seen message when chat is opened
  const handleChatOpen = () => {
    if (onCollapsedChange) {
      onCollapsedChange(false);
    }
    const maxId = Math.max(...messages.map(msg => msg.id), 0);
    setLastSeenMessageId(maxId);
  };

  const handleRoomClick = () => {
    setActiveDropdown(activeDropdown === 'room' ? null : 'room');
  };

  const handleThemeClick = () => {
    setActiveDropdown(activeDropdown === 'theme' ? null : 'theme');
  };

  const handleRoomSelect = (roomId: string) => {
    setIsDropdownOpen(false);
    if (roomId !== 'logs') {
      onUserInput?.('Error: Access restricted. Translation matrix required.');
      return;
    }
    const newRoom = `/${roomId}`;
    onRoomChange?.(newRoom);
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleInputSubmit = async (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && input.trim()) {
      setIsProcessing(true);
      try {
        await onUserInput?.(input.trim());
      } finally {
        setIsProcessing(false);
        setInput('');
      }
    }
  };

  const formatMessage = (text: string, type: ChatMessage['type'], metadata?: ChatMessage['metadata']) => {
    if (type === 'assistant') {
      return <p className={styles.assistant}>{text}</p>;
    }

    // For formatted messages (those containing our ASCII art boxes), convert to a themed list
    if (text.includes('╭') || text.includes('╰')) {
      // Extract the content between the box lines
      const lines = text.split('\n');
      const contentLines = lines.filter(line => {
        const trimmed = line.trim();
        return trimmed && 
               !trimmed.includes('╭') && 
               !trimmed.includes('╮') && 
               !trimmed.includes('╯') && 
               !trimmed.includes('╰') && 
               !trimmed.includes('─') &&
               trimmed.includes('│');
      });
      
      // If we have content, render it as a list
      if (contentLines.length > 0) {
        return (
          <div className={styles.statusList}>
            {contentLines.map((line, index) => {
              // Split by the vertical bar and clean up
              const parts = line.split('│').filter(part => part.trim());
              if (parts.length < 2) return null;
              
              const label = parts[0].trim();
              const value = parts[1].trim();
              
              if (!label) return null;
              return (
                <div key={index} className={styles.statusItem}>
                  <span className={styles.statusLabel}>{label}</span>
                  <span className={styles.statusValue}>{value}</span>
                </div>
              );
            })}
          </div>
        );
      }
      
      // If no content was found, fall back to the original text
      return <pre className={styles.formattedOutput}>{text}</pre>;
    }

    const parts = text.split(': ');
    if (parts.length >= 2) {
      const prefix = parts[0];
      const content = parts.slice(1).join(': ');
      if (prefix.startsWith('System') || prefix === 'Error') {
        return (
          <p>
            <span className={styles.system}>{prefix}: </span>
            {content}
            {metadata && renderMetadata(metadata)}
          </p>
        );
      }
    }
    return <p>{text}</p>;
  };

  const renderMetadata = (metadata: NonNullable<ChatMessage['metadata']>) => {
    const elements = [];
    
    if (metadata.memoryCard) {
      elements.push(
        <div key="memory" className={styles.metadata}>
          <span className={styles.label}>Memory:</span>
          {metadata.memoryCard.features?.map(f => (
            <span key={f} className={styles.tag}>{f}</span>
          ))}
        </div>
      );
    }

    if (metadata.walletHealth) {
      elements.push(
        <div key="health" className={styles.metadata}>
          <span className={styles.label}>Risk Score:</span>
          <span className={`${styles.score} ${getRiskClass(metadata.walletHealth.riskScore)}`}>
            {metadata.walletHealth.riskScore}
          </span>
        </div>
      );
    }

    return elements.length ? <div className={styles.metadataContainer}>{elements}</div> : null;
  };

  const getRiskClass = (score?: number) => {
    if (!score) return '';
    if (score < 0.3) return styles.low;
    if (score < 0.7) return styles.medium;
    return styles.high;
  };

  const handleThemeSelect = (themeId: string) => {
    const selectedTheme = themes.find(t => t.id === themeId);
    if (selectedTheme && selectedTheme.available) {
      if (onRoomChange) {
        onRoomChange(`/theme/${themeId}`);
      }
    }
  };

  const handleEnvelopeClick = () => {
    setShowInbox(true);
  };

  const handleInboxClose = () => {
    setShowInbox(false);
  };

  return isCollapsed ? (
    <button 
      className={`${styles.collapsedButton} ${styles[theme]} ${isEchoActive ? styles.withEcho : ''} ${hasNewActionMessages ? styles.hasNotification : ''}`}
      onClick={handleChatOpen}
    >
      E.C.H.O
    </button>
  ) : (
    <div className={`${styles.chatContainer} ${styles[theme]} ${isEchoActive ? styles.withEcho : ''}`}>
      <div className={styles.hudWindow}>
        <div className={styles.chatHeader}>
          <div className={styles.buttonContainer}>
            <div className={styles.envelopeContainer}>
              <img 
                src={envelopeIcon} 
                alt="Messages" 
                className={styles.envelopeIcon} 
                onClick={handleEnvelopeClick}
              />
            </div>
            <div className={styles.roomSelector} ref={dropdownRef}>
              <button 
                className={styles.roomButton}
                onClick={handleRoomClick}
              >
                [ {currentRoom} ]
              </button>
              {activeDropdown === 'room' && (
                <div className={styles.dropdownContent}>
                  <div className={styles.dropdownSection}>
                    <div className={styles.sectionTitle}>Rooms</div>
                    {chatRooms.map(room => (
                      <button
                        key={room.id}
                        className={`${styles.roomOption} ${!room.available ? styles.disabled : ''}`}
                        onClick={() => {
                          handleRoomSelect(room.id);
                          setActiveDropdown(null);
                        }}
                        disabled={!room.available}
                      >
                        {room.name}
                      </button>
                    ))}
                  </div>
                </div>
              )}
              <button 
                className={styles.themeButton}
                onClick={handleThemeClick}
              >
                [ /{theme} ]
              </button>
              {activeDropdown === 'theme' && (
                <div className={styles.dropdownContent}>
                  <div className={styles.dropdownSection}>
                    <div className={styles.sectionTitle}>Themes</div>
                    {themes.map(t => (
                      <button
                        key={t.id}
                        className={`${styles.themeOption} ${t.id === theme ? styles.active : ''} ${!t.available ? styles.disabled : ''}`}
                        onClick={() => {
                          handleThemeSelect(t.id);
                          setActiveDropdown(null);
                        }}
                        disabled={!t.available}
                      >
                        {t.name}
                        <span className={styles.themeDescription}>{t.description}</span>
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>
            <button 
              className={styles.toggleButton}
              onClick={() => onCollapsedChange?.(true)}
            >
              [ /COLLAPSE ]
            </button>
          </div>
        </div>
        <div className={styles.messageList}>
          {messages.map((message) => (
            <div
              key={message.id}
              className={`${styles.messageItem} ${styles[message.type]} ${isDigitizing ? styles.digitizing : ''}`}
            >
              {message.type === 'action' ? (
                <button 
                  onClick={message.action}
                  className={styles.actionButton}
                >
                  {message.actionText || message.text}
                </button>
              ) : (
                formatMessage(message.text, message.type, message.metadata)
              )}
            </div>
          ))}
          <div ref={messagesEndRef} />
        </div>
        <div className={styles.inputContainer}>
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleInputSubmit}
            placeholder={isProcessing ? "Processing..." : "Enter command..."}
            spellCheck={false}
            disabled={isProcessing}
          />
          <span className={styles.invalidText}>matrix invalid</span>
        </div>
      </div>
      {showInbox && (
        <div className={styles.inboxContainer}>
          <div className={styles.inboxHeader}>
            <h3>Messages</h3>
            <button className={styles.closeButton} onClick={handleInboxClose}>×</button>
          </div>
          <div className={styles.inboxMessageList}>
            <div className={`${styles.inboxMessageItem} ${styles.unread}`}>
              <div className={styles.messageHeader}>
                <h4 className={styles.messageTitle}>System Update</h4>
                <span className={styles.messageTime}>{new Date().toLocaleTimeString()}</span>
              </div>
              <p className={styles.messagePreview}>New features have been added to the interface.</p>
            </div>
            <div className={styles.inboxMessageItem}>
              <div className={styles.messageHeader}>
                <h4 className={styles.messageTitle}>Connection Status</h4>
                <span className={styles.messageTime}>{new Date(Date.now() - 3600000).toLocaleTimeString()}</span>
              </div>
              <p className={styles.messagePreview}>Your connection to the network is stable.</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default SystemChat; 