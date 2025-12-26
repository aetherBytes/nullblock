import React, { useState, useRef, useEffect, useCallback } from 'react';
import { agentService } from '../../../common/services/agent-service';
import MarkdownRenderer from '../../common/MarkdownRenderer';
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
  onUserMessageSent?: (messageId: string) => void;
  onAgentResponseReceived?: (messageId: string) => void;
  tendrilHit?: boolean;
}

// Energy state for transmission animation
type EnergyState = 'idle' | 'charging' | 'firing' | 'processing';

const VoidChatHUD: React.FC<VoidChatHUDProps> = ({
  publicKey: _publicKey,
  isActive = true,
  onFirstMessage,
  onUserMessageSent,
  onAgentResponseReceived,
  tendrilHit = false,
}) => {
  const [input, setInput] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [energyState, setEnergyState] = useState<EnergyState>('idle');
  const [messages, setMessages] = useState<VoidMessage[]>([]);
  const [hasInteracted, setHasInteracted] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const [hasUnreadMessages, setHasUnreadMessages] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const historyRef = useRef<HTMLDivElement>(null);
  const pendingMessageRef = useRef<{ message: string; msgId: string } | null>(null);

  // Handle the actual API call after charging/firing animation
  const executeTransmission = useCallback(async () => {
    const pending = pendingMessageRef.current;
    if (!pending) return;

    try {
      const response = await agentService.chatWithAgent('hecate', pending.message);

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
  }, [onAgentResponseReceived]);

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

    if (!hasInteracted) {
      setHasInteracted(true);
      onFirstMessage?.();
    }

    // Store pending message for after animation
    pendingMessageRef.current = { message: userMessage, msgId: userMsg.id };

    // Start charging animation sequence
    setEnergyState('charging');

    // After charging (0.8s), switch to firing
    setTimeout(() => {
      setEnergyState('firing');

      // After firing flash (0.3s), launch tendril and start processing
      setTimeout(() => {
        // Fire the tendril
        onUserMessageSent?.(userMsg.id);
        setIsProcessing(true);
        setEnergyState('processing');

        // Execute the actual API call
        executeTransmission();
      }, 300);
    }, 800);
  }, [input, isProcessing, energyState, hasInteracted, onFirstMessage, onUserMessageSent, executeTransmission]);

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

  // Trigger notification glow when tendril hits (if history is closed)
  useEffect(() => {
    if (tendrilHit && !showHistory && messages.length > 0) {
      setHasUnreadMessages(true);
    }
  }, [tendrilHit, showHistory, messages.length]);

  if (!isActive) return null;

  return (
    <div className={styles.voidChatContainer}>
      {/* Input bar */}
      <div className={styles.voidInputBar}>
        {/* Chat History Popup */}
        {showHistory && (
          <div className={styles.historyPopup}>
            <div className={styles.historyHeader}>
              <span className={styles.historyTitle}>Transmission Log</span>
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
                messages.map((msg) => (
                  <div
                    key={msg.id}
                    className={`${styles.historyMessage} ${msg.sender === 'user' ? styles.historyUser : styles.historyAgent}`}
                  >
                    <div className={styles.historyMeta}>
                      <span className={styles.historySender}>
                        {msg.sender === 'user' ? 'You' : 'HECATE'}
                      </span>
                      <span className={styles.historyTime}>
                        {msg.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                      </span>
                    </div>
                    <div className={styles.historyText}>
                      <MarkdownRenderer content={msg.text} />
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}
        <form onSubmit={handleSubmit} className={styles.inputForm}>
          <div className={`${styles.inputContainer} ${energyState === 'charging' ? styles.charging : ''} ${energyState === 'firing' ? styles.firing : ''} ${energyState === 'processing' ? styles.processing : ''} ${tendrilHit ? styles.tendrilHit : ''}`}>
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
                  }
                }
              }}
              aria-label="Toggle chat history"
              disabled={messages.length === 0}
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M12 8v4l3 3" />
                <circle cx="12" cy="12" r="10" />
              </svg>
            </button>
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={energyState === 'processing' ? 'Awaiting transmission response...' : 'Transmit to HECATE...'}
              className={styles.voidInput}
              disabled={energyState !== 'idle'}
              rows={1}
            />
            <button
              type="submit"
              className={styles.sendButton}
              disabled={energyState !== 'idle' || !input.trim()}
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
