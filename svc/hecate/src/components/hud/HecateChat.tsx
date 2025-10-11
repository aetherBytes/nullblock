import React, { useRef, useEffect } from 'react';
import MarkdownRenderer from '../common/MarkdownRenderer';
import styles from './hud.module.scss';

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

interface HecateChatProps {
  chatMessages: ChatMessage[];
  chatInput: string;
  setChatInput: (value: string) => void;
  chatInputRef: React.RefObject<HTMLTextAreaElement>;
  chatMessagesRef: React.RefObject<HTMLDivElement>;
  chatEndRef: React.RefObject<HTMLDivElement>;
  nullviewState: string;
  isModelChanging: boolean;
  isProcessingChat: boolean;
  defaultModelReady: boolean;
  currentSelectedModel: string | null;
  agentHealthStatus?: 'healthy' | 'unhealthy' | 'unknown';
  isChatExpanded: boolean;
  setIsChatExpanded: (expanded: boolean) => void;
  isScopesExpanded: boolean;
  setIsScopesExpanded: (expanded: boolean) => void;
  activeScope: string | null;
  setActiveLens: (scope: string | null) => void;
  onChatSubmit: (e: React.FormEvent) => void;
  onChatInputChange: (e: React.ChangeEvent<HTMLTextAreaElement>) => void;
  onChatScroll: (e: React.UIEvent<HTMLDivElement>) => void;
  scrollToBottom: () => void;
  isUserScrolling: boolean;
  chatAutoScroll: boolean;
  activeAgent?: 'hecate' | 'siren';
  setActiveAgent?: (agent: 'hecate' | 'siren') => void;
  getImagesForMessage: (messageId: string) => ImageData[];
}

const ChatMessageComponent = React.memo<{
  message: ChatMessage;
  getImagesForMessage: (messageId: string) => ImageData[];
  handleAgentSwitch: (agent: 'hecate' | 'siren') => void;
}>(({ message, getImagesForMessage, handleAgentSwitch }) => {
  const images = getImagesForMessage(message.id);

  return (
    <div
      className={`${styles.chatMessage} ${styles[`message-${message.sender}`]} ${message.type ? styles[`type-${message.type}`] : ''}`}
    >
      <div className={styles.messageHeader}>
        <span className={styles.messageSender}>
          {message.sender === 'hecate' || message.sender === 'siren' ? (
            <span className={message.sender === 'siren' ? styles.sirenMessageSender : styles.hecateMessageSender}>
              <span
                className={styles.clickableAgentName}
                onClick={() => handleAgentSwitch(message.sender)}
                title={`Switch to ${message.sender === 'hecate' ? 'Hecate' : 'Siren'} agent`}
              >
                🤖 {message.sender === 'hecate' ? 'Hecate' : message.sender === 'siren' ? 'Siren' : 'Agent'}
              </span>
            </span>
          ) : (
            '👤 You'
          )}
        </span>
        <span className={styles.messageTime}>
          {message.timestamp.toLocaleTimeString()}
        </span>
      </div>
      <div className={styles.messageContent}>
        {message.isTaskResult && (
          <div className={styles.taskResultHeader}>
            <div className={styles.taskResultBadge}>
              <span className={styles.taskIcon}>✅</span>
              <span className={styles.taskLabel}>Task Result</span>
              {message.taskName && <span className={styles.taskName}>"{message.taskName}"</span>}
            </div>
            {message.processingTime && (
              <div className={styles.taskMetrics}>
                <span className={styles.processingTime}>⏱️ {message.processingTime}ms</span>
              </div>
            )}
          </div>
        )}
        <MarkdownRenderer
          content={message.message}
          images={images}
        />
      </div>
    </div>
  );
}, (prevProps, nextProps) => {
  return (
    prevProps.message.id === nextProps.message.id &&
    prevProps.message.message === nextProps.message.message &&
    prevProps.message.type === nextProps.message.type
  );
});

ChatMessageComponent.displayName = 'ChatMessageComponent';

const HecateChat: React.FC<HecateChatProps> = ({
  chatMessages,
  chatInput,
  setChatInput,
  chatInputRef,
  chatMessagesRef,
  chatEndRef,
  nullviewState,
  isModelChanging,
  isProcessingChat,
  defaultModelReady,
  currentSelectedModel,
  agentHealthStatus = 'unknown',
  isChatExpanded,
  setIsChatExpanded,
  isScopesExpanded,
  setIsScopesExpanded,
  activeScope,
  setActiveLens,
  onChatSubmit,
  onChatInputChange,
  onChatScroll,
  scrollToBottom,
  isUserScrolling,
  chatAutoScroll,
  activeAgent = 'hecate',
  setActiveAgent,
  getImagesForMessage
}) => {

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      onChatSubmit(e);
    }
  };

  const handleAgentSwitch = (agentName: 'hecate' | 'siren') => {
    if (setActiveAgent && agentName !== activeAgent) {
      console.log(`🔄 Switching to ${agentName} agent from chat`);
      setActiveAgent(agentName);
    }
  };

  const handleTextareaResize = () => {
    if (chatInputRef.current) {
      chatInputRef.current.style.height = 'auto';
      const scrollHeight = chatInputRef.current.scrollHeight;
      const maxHeight = 200; // Maximum height in pixels
      chatInputRef.current.style.height = `${Math.min(scrollHeight, maxHeight)}px`;
    }
  };

  useEffect(() => {
    handleTextareaResize();
  }, [chatInput]);

  useEffect(() => {
    if (chatInputRef.current) {
      chatInputRef.current.addEventListener('input', handleTextareaResize);
      return () => {
        if (chatInputRef.current) {
          chatInputRef.current.removeEventListener('input', handleTextareaResize);
        }
      };
    }
  }, []);
  const renderThinkingIndicator = () => (
    <div className={`${styles.chatMessage} ${styles['message-hecate']} ${styles['type-thinking']}`}>
      <div className={styles.messageHeader}>
        <span className={styles.messageSender}>
          <span className={styles.hecateMessageSender}>
            <span
              className={styles.clickableAgentName}
              onClick={() => handleAgentSwitch(activeAgent === 'hecate' ? 'siren' : 'hecate')}
              title={`Switch to ${activeAgent === 'hecate' ? 'Siren' : 'Hecate'} agent`}
            >
              🤖 {activeAgent || 'Hecate'}
            </span>
          </span>
        </span>
        <span className={styles.messageTime}>
          {new Date().toLocaleTimeString()}
        </span>
      </div>
      <div className={styles.messageContent}>
        <div className={styles.thinkingIndicator}>
          <span className={styles.thinkingDots}>●</span>
          <span className={styles.thinkingDots}>●</span>
          <span className={styles.thinkingDots}>●</span>
          <span className={styles.thinkingText}>{activeAgent || 'Hecate'} is thinking...</span>
        </div>
      </div>
    </div>
  );

  const renderChatMessage = (message: ChatMessage) => (
    <ChatMessageComponent
      key={message.id}
      message={message}
      getImagesForMessage={getImagesForMessage}
      handleAgentSwitch={handleAgentSwitch}
    />
  );

  const expandedChatContent = (
    <div className={styles.fullscreenOverlay}>
      <div className={`${styles.chatSection} ${styles.expanded}`}>
        <div className={styles.hecateChat}>
          <div className={styles.chatHeader}>
            <div className={styles.chatTitle}>
              <h4>💬 <span
                className={styles.clickableAgentName}
                onClick={() => handleAgentSwitch(activeAgent === 'hecate' ? 'siren' : 'hecate')}
                title={`Switch to ${activeAgent === 'hecate' ? 'Siren' : 'Hecate'} agent`}
              >
                {activeAgent || 'Hecate'}
              </span> Chat</h4>
              <span className={styles.modelStatus}>
                {agentHealthStatus === 'unhealthy' ? '⚠️ API Keys Required' :
                 defaultModelReady || currentSelectedModel ? 'Ready' : 'Loading...'}
              </span>
            </div>
            <div className={styles.chatHeaderControls}>
              <button
                className={styles.expandButton}
                onClick={() => {
                  setIsChatExpanded(false);
                  if (activeScope) setActiveLens(null);
                }}
                title="Exit full screen"
              >
                ⊟
              </button>
            </div>
          </div>

          <div className={styles.chatMessages} ref={chatMessagesRef} onScroll={onChatScroll}>
            {chatMessages.map(renderChatMessage)}
            {nullviewState === 'thinking' && renderThinkingIndicator()}
            <div ref={chatEndRef} />
          </div>

          <form className={styles.chatInput} onSubmit={onChatSubmit}>
            <div className={styles.chatInputContainer}>
              <textarea
                ref={chatInputRef}
                value={chatInput}
                onChange={onChatInputChange}
                onKeyDown={handleKeyDown}
                placeholder={
                  agentHealthStatus === 'unhealthy'
                    ? "⚠️ Configure API keys in settings first..."
                    : isModelChanging
                      ? "Switching models..."
                      : isProcessingChat || nullviewState === 'thinking'
                        ? "Hecate is thinking..."
                        : "Ask Hecate anything... (Enter to send, Shift+Enter for new line)"
                }
                className={styles.chatInputField}
                disabled={agentHealthStatus === 'unhealthy' || isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)}
                rows={1}
              />
              <button
                type="submit"
                className={styles.chatSendButton}
                disabled={agentHealthStatus === 'unhealthy' || isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)}
              >
                <span>➤</span>
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );

  const regularChatContent = (
    <div className={`${styles.chatSection} ${isChatExpanded ? styles.hidden : ''} ${isScopesExpanded ? styles.hidden : ''}`}>
      <div className={styles.hecateChat}>
        <div className={styles.chatHeader}>
          <div className={styles.chatTitle}>
            <h4>
              <span
                className={styles.clickableAgentName}
                onClick={() => handleAgentSwitch(activeAgent === 'hecate' ? 'siren' : 'hecate')}
                title={`Switch to ${activeAgent === 'hecate' ? 'Siren' : 'Hecate'} agent`}
              >
                {(activeAgent || 'HECATE').toUpperCase()}
              </span>
              :{currentSelectedModel ? currentSelectedModel.split('/').pop()?.split(':')[0]?.toUpperCase() || 'MODEL' : 'LOADING'}
            </h4>
            <span className={styles.chatStatus}>
              {agentHealthStatus === 'unhealthy' ? '⚠️ API Keys Required' :
               defaultModelReady || currentSelectedModel ? 'Ready' : 'Loading...'}
            </span>
          </div>
          <div className={styles.chatHeaderControls}>
            <button
              className={styles.expandButton}
              onClick={() => {
                const newChatExpanded = !isChatExpanded;
                setIsChatExpanded(newChatExpanded);
                if (isScopesExpanded) setIsScopesExpanded(false);
                if (newChatExpanded && activeScope) setActiveLens(null);
              }}
              title={isChatExpanded ? "Exit full screen" : "Expand chat full screen"}
            >
              {isChatExpanded ? '⊟' : '⊞'}
            </button>
          </div>
        </div>

        <div className={styles.chatMessages} ref={chatMessagesRef} onScroll={onChatScroll}>
          {chatMessages.map(renderChatMessage)}
          {nullviewState === 'thinking' && renderThinkingIndicator()}
          <div ref={chatEndRef} />
          
          {/* Scroll to bottom button - show when user has scrolled up or when there are messages */}
          {(isUserScrolling || chatMessages.length > 0) && (
            <button
              className={styles.scrollToBottomButton}
              onClick={scrollToBottom}
              title="Scroll to bottom"
            >
              ↓
            </button>
          )}
        </div>

        <form className={styles.chatInput} onSubmit={onChatSubmit}>
          <div className={styles.chatInputContainer}>
            <textarea
              ref={chatInputRef}
              value={chatInput}
              onChange={onChatInputChange}
              onKeyDown={handleKeyDown}
              placeholder={
                agentHealthStatus === 'unhealthy'
                  ? "⚠️ Configure API keys in settings first..."
                  : isModelChanging
                    ? "Switching models..."
                    : isProcessingChat || nullviewState === 'thinking'
                      ? `${activeAgent || 'Hecate'} is thinking...`
                      : `Ask ${activeAgent || 'Hecate'} anything... (Enter to send, Shift+Enter for new line)`
              }
              className={styles.chatInputField}
              disabled={agentHealthStatus === 'unhealthy' || isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)}
              rows={1}
            />
            <button
              type="submit"
              className={styles.chatSendButton}
              disabled={agentHealthStatus === 'unhealthy' || isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)}
            >
              <span>➤</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );

  return (
    <>
      {isChatExpanded && expandedChatContent}
      {regularChatContent}
    </>
  );
};

export default HecateChat;