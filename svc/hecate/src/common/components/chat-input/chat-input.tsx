import React, { useState } from 'react';
import styles from './styles.module.scss';

const ChatInput = () => {
  const [message, setMessage] = useState('');

  const sendMessage = (e) => {
    if (e.key === 'Enter' && message.trim()) {
      // Trigger the float animation for the message
      const messageContainer = document.createElement('div');
      messageContainer.className = styles.floatingMessage; // Use styles from module
      messageContainer.textContent = message;
      document.body.appendChild(messageContainer);

      // Reset message input
      setMessage('');

      // Remove the element after animation completes (adjust time as needed)
      setTimeout(() => {
        document.body.removeChild(messageContainer);
      }, 3000);
    }
  };

  return (
    <div className={styles.chatInputContainer}>
      <input
        type="text"
        placeholder="Type your message..."
        value={message}
        onChange={(e) => setMessage(e.target.value)}
        onKeyPress={sendMessage}
        className={styles.chatInput}
      />
    </div>
  );
};

export default ChatInput;

