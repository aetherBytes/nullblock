import React from 'react';
import styles from './system-chat.module.scss';

interface ChatMessage {
  id: number;
  text: string;
  type: 'message' | 'alert' | 'critical' | 'upgrade' | 'action';
  action?: () => void;
  actionText?: string;
}

interface SystemChatProps {
  messages: ChatMessage[];
}

const SystemChat: React.FC<SystemChatProps> = ({ messages }) => {
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
    <div className={styles.chatContainer}>
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
      </div>
    </div>
  );
};

export default SystemChat; 