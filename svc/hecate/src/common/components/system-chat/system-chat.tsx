import React, { useState, useRef, useEffect } from 'react';
import styles from './system-chat.module.scss';

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
  isDigitizing = false
}) => {
  const [input, setInput] = useState('');
  const [isFullScreen, setIsFullScreen] = useState(false);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [lastSeenMessageId, setLastSeenMessageId] = useState(0);
  const [isProcessing, setIsProcessing] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const chatRooms = [
    { id: 'logs', name: '/logs', available: true },
    { id: 'memory', name: '/memory', available: false },
    { id: 'health', name: '/health', available: false },
    { id: 'reality', name: '/reality', available: false }
  ];

  // Update collapsed state when prop changes
  useEffect(() => {
    if (onCollapsedChange) {
      onCollapsedChange(isCollapsed);
    }
  }, [isCollapsed]);

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

    // For formatted messages (those containing our ASCII art boxes), preserve formatting
    if (text.includes('╭') || text.includes('╰')) {
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

  return isCollapsed ? (
    <button 
      className={`${styles.collapsedButton} ${isEchoActive ? styles.withEcho : ''} ${hasNewActionMessages ? styles.hasNotification : ''}`}
      onClick={handleChatOpen}
    >
      [ ECHO ]
    </button>
  ) : (
    <div className={`${styles.chatContainer} ${isEchoActive ? styles.withEcho : ''} ${isFullScreen ? styles.fullScreen : ''}`}>
      <div className={styles.hudWindow}>
        <div className={styles.chatHeader}>
          <div className={styles.roomSelector}>
            <button 
              className={styles.dropdownButton}
              onClick={() => setIsDropdownOpen(!isDropdownOpen)}
            >
              [ {currentRoom} ]
            </button>
            {isDropdownOpen && (
              <div className={styles.dropdownContent}>
                {chatRooms.map(room => (
                  <button
                    key={room.id}
                    className={`${styles.roomOption} ${!room.available ? styles.disabled : ''}`}
                    onClick={() => handleRoomSelect(room.id)}
                  >
                    {room.name} {!room.available && '(locked)'}
                  </button>
                ))}
              </div>
            )}
          </div>
          <div className={styles.controls}>
            <button 
              className={styles.toggleButton}
              onClick={() => onCollapsedChange?.(true)}
            >
              [ Collapse ]
            </button>
            <button 
              className={styles.toggleButton}
              onClick={() => setIsFullScreen(!isFullScreen)}
            >
              {isFullScreen ? '[ Minimize ]' : '[ Expand ]'}
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
          <span className={styles.invalidText}>translation matrix invalid</span>
        </div>
      </div>
    </div>
  );
};

export default SystemChat; 