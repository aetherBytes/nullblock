import React, { useState, useRef, useEffect } from 'react';
import styles from './system-chat.module.scss';

interface ChatMessage {
  id: number;
  text: string;
  type: 'message' | 'alert' | 'critical' | 'update' | 'action' | 'user';
  action?: () => void;
  actionText?: string;
}

interface SystemChatProps {
  messages: ChatMessage[];
  isEchoActive?: boolean;
  onUserInput?: (input: string) => void;
}

const SystemChat: React.FC<SystemChatProps> = ({ messages, isEchoActive = false, onUserInput }) => {
  const [input, setInput] = useState('');
  const [isFullScreen, setIsFullScreen] = useState(false);
  const [isCollapsed, setIsCollapsed] = useState(true);
  const [selectedRoom, setSelectedRoom] = useState('/logs');
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const [hasSeenEcho, setHasSeenEcho] = useState(false);
  const [lastSeenMessageId, setLastSeenMessageId] = useState(0);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Check if there are any new pending action messages
  const hasNewActionMessages = messages.some(msg => 
    msg.type === 'action' && msg.id > lastSeenMessageId
  );

  // Update last seen message when chat is opened
  const handleChatOpen = () => {
    setIsCollapsed(false);
    const maxId = Math.max(...messages.map(msg => msg.id), 0);
    setLastSeenMessageId(maxId);
  };

  useEffect(() => {
    // Load last state from localStorage
    const lastCollapsedState = localStorage.getItem('chatCollapsedState');
    const lastSeenEcho = localStorage.getItem('hasSeenEcho');
    
    if (isEchoActive && !lastSeenEcho) {
      // First time seeing ECHO, force collapse
      setIsCollapsed(true);
      setHasSeenEcho(true);
      localStorage.setItem('hasSeenEcho', 'true');
    } else if (lastCollapsedState) {
      // Use last saved state
      setIsCollapsed(lastCollapsedState === 'true');
    }
  }, [isEchoActive]);

  // Save collapsed state when it changes
  useEffect(() => {
    localStorage.setItem('chatCollapsedState', isCollapsed.toString());
  }, [isCollapsed]);

  const chatRooms = [
    { id: 'logs', name: '/logs', available: true },
    { id: 'agents', name: '/agents', available: false },
    { id: 'camp', name: '/camp', available: false },
    { id: 'reality', name: '/reality', available: false }
  ];

  const handleRoomSelect = (roomId: string) => {
    setIsDropdownOpen(false);
    if (roomId !== 'logs') {
      onUserInput?.('Error: Access restricted.');
      return;
    }
    setSelectedRoom(`/${roomId}`);
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleInputSubmit = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && input.trim()) {
      onUserInput?.(input.trim());
      setInput('');
    }
  };

  const formatMessage = (text: string, type: ChatMessage['type']) => {
    const parts = text.split(': ');
    if (parts.length >= 2 && parts[0].startsWith('System')) {
      const prefix = parts[0];
      const content = parts.slice(1).join(': ');
      return (
        <p>
          <span className={styles.system}>{prefix}: </span>
          {content}
        </p>
      );
    }
    return <p>{text}</p>;
  };

  return isCollapsed ? (
    <button 
      className={`${styles.collapsedButton} ${isEchoActive ? styles.withEcho : ''} ${hasNewActionMessages ? styles.hasNotification : ''}`}
      onClick={handleChatOpen}
    >
      [ CHAT ]
    </button>
  ) : (
    <div className={`${styles.chatContainer} ${isEchoActive ? styles.withEcho : ''} ${isFullScreen ? styles.fullScreen : ''}`}>
      <div className={styles.chatHeader}>
        <div className={styles.roomSelector}>
          <button 
            className={styles.dropdownButton}
            onClick={() => setIsDropdownOpen(!isDropdownOpen)}
          >
            [ {selectedRoom} ]
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
            onClick={() => setIsCollapsed(true)}
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
            className={`${styles.messageItem} ${styles[message.type]}`}
          >
            {message.type === 'action' ? (
              <button 
                onClick={message.action}
                className={styles.actionButton}
              >
                {message.actionText || message.text}
              </button>
            ) : (
              formatMessage(message.text, message.type)
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
          placeholder="Enter command..."
          spellCheck={false}
          disabled={true}
        />
        <span className={styles.invalidText}>translation matrix invalid</span>
      </div>
    </div>
  );
};

export default SystemChat; 