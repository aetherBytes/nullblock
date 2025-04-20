import React from 'react';
import styles from './inbox.module.scss';

export interface Message {
  id: string;
  title: string;
  content: string;
  timestamp: Date;
  unread: boolean;
}

interface InboxProps {
  messages: Message[];
  onClose: () => void;
  onMessageClick: (message: Message) => void;
}

const Inbox: React.FC<InboxProps> = ({ messages, onClose, onMessageClick }) => {
  return (
    <div className={styles.inboxContainer}>
      <div className={styles.inboxHeader}>
        <h3>Messages</h3>
        <button className={styles.closeButton} onClick={onClose}>×</button>
      </div>
      <div className={styles.messageList}>
        {messages.map((message) => (
          <div 
            key={message.id} 
            className={`${styles.messageItem} ${message.unread ? styles.unread : ''}`}
            onClick={() => onMessageClick(message)}
          >
            <div className={styles.messageHeader}>
              <h4 className={styles.messageTitle}>{message.title}</h4>
              <span className={styles.messageTime}>
                {message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
              </span>
            </div>
            <p className={styles.messagePreview}>{message.content}</p>
          </div>
        ))}
      </div>
    </div>
  );
};

export default Inbox; 