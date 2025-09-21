import React, { useRef, useEffect } from 'react';
import MarkdownRenderer from '../common/MarkdownRenderer';
import styles from './hud.module.scss';

interface ChatMessage {
  id: string;
  timestamp: Date;
  sender: 'user' | 'hecate';
  message: string;
  type?: string;
  model_used?: string;
  metadata?: any;
}

interface HecateChatProps {
  chatMessages: ChatMessage[];
  chatInput: string;
  setChatInput: (value: string) => void;
  chatInputRef: React.RefObject<HTMLInputElement>;
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
  onChatInputChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onChatScroll: (e: React.UIEvent<HTMLDivElement>) => void;
}

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
  onChatScroll
}) => {
  const renderThinkingIndicator = () => (
    <div className={`${styles.chatMessage} ${styles['message-hecate']} ${styles['type-thinking']}`}>
      <div className={styles.messageHeader}>
        <span className={styles.messageSender}>
          <span className={styles.hecateMessageSender}>
            <div className={`${styles.nullviewChat} ${styles['chat-thinking']} ${styles.clickableNulleyeChat}`}>
              <div className={styles.staticFieldChat}></div>
              <div className={styles.coreNodeChat}></div>
              <div className={styles.streamLineChat}></div>
              <div className={styles.lightningSparkChat}></div>
            </div>
            Hecate
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
          <span className={styles.thinkingText}>Thinking...</span>
        </div>
      </div>
    </div>
  );

  const renderChatMessage = (message: ChatMessage) => (
    <div
      key={message.id}
      className={`${styles.chatMessage} ${styles[`message-${message.sender}`]} ${message.type ? styles[`type-${message.type}`] : ''}`}
    >
      <div className={styles.messageHeader}>
        <span className={styles.messageSender}>
          {message.sender === 'hecate' ? (
            <span className={styles.hecateMessageSender}>
              <div className={`${styles.nullviewChat} ${styles[`chat-${nullviewState === 'thinking' ? 'thinking' : (message.type || 'base')}`]} ${styles.clickableNulleyeChat}`}>
                <div className={styles.staticFieldChat}></div>
                <div className={styles.coreNodeChat}></div>
                <div className={styles.streamLineChat}></div>
                <div className={styles.lightningSparkChat}></div>
              </div>
              Hecate
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
        <MarkdownRenderer content={message.message} />
      </div>
    </div>
  );

  const expandedChatContent = (
    <div className={styles.fullscreenOverlay}>
      <div className={`${styles.chatSection} ${styles.expanded}`}>
        <div className={styles.hecateChat}>
          <div className={styles.chatHeader}>
            <div className={styles.chatTitle}>
              <h4>💬 Hecate Chat</h4>
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
            <input
              ref={chatInputRef}
              type="text"
              value={chatInput}
              onChange={onChatInputChange}
              placeholder={
                agentHealthStatus === 'unhealthy'
                  ? "⚠️ Configure API keys in settings first..."
                  : isModelChanging
                    ? "Switching models..."
                    : isProcessingChat || nullviewState === 'thinking'
                      ? "Hecate is thinking..."
                      : "Ask Hecate anything..."
              }
              className={styles.chatInputField}
              disabled={agentHealthStatus === 'unhealthy' || isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)}
            />
            <button
              type="submit"
              className={styles.chatSendButton}
              disabled={agentHealthStatus === 'unhealthy' || isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)}
            >
              <span>➤</span>
            </button>
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
            <h4>{currentSelectedModel ? `HECATE:${currentSelectedModel.split('/').pop()?.split(':')[0]?.toUpperCase() || 'MODEL'}` : 'HECATE:LOADING'}</h4>
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
        </div>

        <form className={styles.chatInput} onSubmit={onChatSubmit}>
          <input
            ref={chatInputRef}
            type="text"
            value={chatInput}
            onChange={onChatInputChange}
            placeholder={
              agentHealthStatus === 'unhealthy'
                ? "⚠️ Configure API keys in settings first..."
                : isModelChanging
                  ? "Switching models..."
                  : isProcessingChat || nullviewState === 'thinking'
                    ? "Hecate is thinking..."
                    : "Ask Hecate anything..."
            }
            className={styles.chatInputField}
            disabled={agentHealthStatus === 'unhealthy' || isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)}
          />
          <button
            type="submit"
            className={styles.chatSendButton}
            disabled={agentHealthStatus === 'unhealthy' || isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)}
          >
            <span>➤</span>
          </button>
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