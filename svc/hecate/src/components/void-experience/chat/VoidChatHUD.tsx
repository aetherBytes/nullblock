import React, { useState, useRef, useEffect, useCallback } from 'react';
import { agentService } from '../../../common/services/agent-service';
import styles from './voidChat.module.scss';

interface VoidMessage {
  id: string;
  text: string;
  sender: 'user' | 'agent';
  timestamp: Date;
}

interface VoidChatHUDProps {
  publicKey: string | null;
  isActive?: boolean;
  onFirstMessage?: () => void;
}

const VoidChatHUD: React.FC<VoidChatHUDProps> = ({
  publicKey: _publicKey,
  isActive = true,
  onFirstMessage,
}) => {
  const [input, setInput] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [messages, setMessages] = useState<VoidMessage[]>([]);
  const [hasInteracted, setHasInteracted] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Recent messages for display (last 3)
  const recentMessages = messages.slice(-3);

  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();

    if (!input.trim() || isProcessing) return;

    const userMessage = input.trim();
    setInput('');
    setIsProcessing(true);

    // Add user message
    const userMsg: VoidMessage = {
      id: `user-${Date.now()}`,
      text: userMessage,
      sender: 'user',
      timestamp: new Date(),
    };
    setMessages(prev => [...prev, userMsg]);

    if (!hasInteracted) {
      setHasInteracted(true);
      onFirstMessage?.();
    }

    try {
      const response = await agentService.chatWithAgent('hecate', userMessage);

      if (response.success && response.data) {
        const agentMsg: VoidMessage = {
          id: `agent-${Date.now()}`,
          text: response.data.content,
          sender: 'agent',
          timestamp: new Date(),
        };
        setMessages(prev => [...prev, agentMsg]);
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
    }
  }, [input, isProcessing, hasInteracted, onFirstMessage]);

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

  if (!isActive) return null;

  return (
    <div className={styles.voidChatContainer}>
      {/* Floating messages */}
      <div className={styles.messagesContainer}>
        {recentMessages.map((msg, index) => (
          <div
            key={msg.id}
            className={`${styles.voidMessage} ${msg.sender === 'user' ? styles.userMessage : styles.agentMessage}`}
            style={{
              animationDelay: `${index * 0.1}s`,
              opacity: 1 - (recentMessages.length - 1 - index) * 0.2,
            }}
          >
            <span className={styles.messageText}>{msg.text}</span>
          </div>
        ))}
        {isProcessing && (
          <div className={`${styles.voidMessage} ${styles.agentMessage} ${styles.thinking}`}>
            <span className={styles.thinkingDots}>
              <span>.</span><span>.</span><span>.</span>
            </span>
          </div>
        )}
      </div>

      {/* Input bar */}
      <div className={styles.voidInputBar}>
        <form onSubmit={handleSubmit} className={styles.inputForm}>
          <div className={`${styles.inputContainer} ${isProcessing ? styles.processing : ''}`}>
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Speak to the void..."
              className={styles.voidInput}
              disabled={isProcessing}
              rows={1}
            />
            <button
              type="submit"
              className={styles.sendButton}
              disabled={isProcessing || !input.trim()}
              aria-label="Send message"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z" />
              </svg>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default VoidChatHUD;
