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
  const messagesEndRef = useRef<HTMLDivElement>(null);

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
    if (parts.length === 2 && parts[0].startsWith('System')) {
      return (
        <p>
          <span className={styles.system}>{parts[0]}: </span>
          {parts[1]}
        </p>
      );
    }
    return <p>{text}</p>;
  };

  return (
    <div className={`${styles.chatContainer} ${isEchoActive ? styles.withEcho : ''}`}>
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
          onKeyPress={handleInputSubmit}
          placeholder="Enter command..."
          spellCheck={false}
        />
      </div>
    </div>
  );
};

export default SystemChat; 